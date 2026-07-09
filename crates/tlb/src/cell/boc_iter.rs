use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::Arc;

use crate::Cell;
use crate::cell::iter::Frame;

/// Iterator over a Cell DAG that yields cells in an order suitable for
/// [Bag of Cells](https://docs.ton.org/blockchain-basics/primitives/serialization/boc#serialization)
/// serialization.
///
/// For `R → [A, B], A → [C], B → [D]` the iterator yields
/// `D, C, B, A, R`. Reversed: `R, A, B, C, D` — a valid topological order.
///
/// # Multi-root
///
/// [`Self::from_roots`] processes roots left-to-right (so the first root's
/// subtree is drained first).
#[derive(Debug, Clone)]
pub struct BagOfCellsIter<'a> {
    stack: Vec<Frame<'a>>,
}

impl<'a> BagOfCellsIter<'a> {
    #[inline]
    pub(crate) fn from_roots<T>(roots: &'a [T]) -> Self
    where
        T: AsRef<Cell>,
    {
        let stack = roots
            .iter()
            .rev()
            .map(AsRef::as_ref)
            .map(Frame::Emit)
            .chain(roots.iter().rev().map(AsRef::as_ref).map(Frame::Visit))
            .collect();

        Self { stack }
    }

    #[inline]
    pub fn augmented<A, F>(self, f: F) -> AugmentedBagOfCellsIter<'a, F, A, EmitAll>
    where
        F: for<'m> FnMut(&'a Cell, ChildrenAugments<'m, A>) -> A,
        A: Clone,
    {
        AugmentedBagOfCellsIter::new(self, f)
    }
}

impl<'a> Iterator for BagOfCellsIter<'a> {
    type Item = &'a Cell;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(frame) = self.stack.pop() {
            match frame {
                Frame::Visit(cell) => self.stack.extend(
                    cell.references
                        .iter()
                        .map(Deref::deref)
                        .map(Frame::Emit)
                        .chain(cell.references.iter().map(Deref::deref).map(Frame::Visit)),
                ),
                Frame::Emit(cell) => return Some(cell),
            };
        }

        None
    }
}

pub struct EmitAll;
pub struct EmitUnique;

#[derive(Debug, Clone)]
pub struct AugmentedBagOfCellsIter<'a, F, A, U = EmitAll> {
    inner: BagOfCellsIter<'a>,
    memo: HashMap<&'a Cell, A>,
    f: F,
    _mode: PhantomData<U>,
}

impl<'a, F, A> AugmentedBagOfCellsIter<'a, F, A, EmitAll> {
    #[inline]
    pub fn new(inner: BagOfCellsIter<'a>, f: F) -> Self {
        Self {
            inner,
            memo: HashMap::new(),
            f,
            _mode: PhantomData,
        }
    }

    #[inline]
    pub fn unique(self) -> AugmentedBagOfCellsIter<'a, F, A, EmitUnique> {
        AugmentedBagOfCellsIter {
            inner: self.inner,
            memo: self.memo,
            f: self.f,
            _mode: PhantomData,
        }
    }
}

impl<'a, F, A, U> AugmentedBagOfCellsIter<'a, F, A, U> {
    #[inline]
    fn compute(&mut self, cell: &'a Cell) -> A
    where
        F: for<'m> FnMut(&'a Cell, ChildrenAugments<'m, A>) -> A,
        A: Clone,
    {
        let children = ChildrenAugments {
            refs: cell.references.iter(),
            memo: &self.memo,
        };
        let value = (self.f)(cell, children);
        self.memo.insert(cell, value.clone());

        value
    }
}

impl<'a, F, A> Iterator for AugmentedBagOfCellsIter<'a, F, A, EmitAll>
where
    F: for<'m> FnMut(&'a Cell, ChildrenAugments<'m, A>) -> A,
    A: Clone,
{
    type Item = (&'a Cell, A);

    fn next(&mut self) -> Option<Self::Item> {
        let cell = self.inner.next()?;
        let value = self.compute(cell);
        Some((cell, value))
    }
}

impl<'a, F, A> Iterator for AugmentedBagOfCellsIter<'a, F, A, EmitUnique>
where
    F: for<'m> FnMut(&'a Cell, ChildrenAugments<'m, A>) -> A,
    A: Clone,
{
    type Item = (&'a Cell, A);

    fn next(&mut self) -> Option<Self::Item> {
        let cell = self.inner.find(|c| !self.memo.contains_key(c))?;
        let value = self.compute(cell);
        Some((cell, value))
    }
}

#[derive(Debug)]
pub(crate) struct ChildrenAugments<'a, A> {
    refs: std::slice::Iter<'a, Arc<Cell>>,
    memo: &'a HashMap<&'a Cell, A>,
}

impl<'a, A> Iterator for ChildrenAugments<'a, A> {
    type Item = &'a A;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.refs.find_map(|c| self.memo.get(c.deref()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::tests::*;

    #[test]
    fn iter_tree_descendants_before_ancestors() {
        let root = make_tree();

        let order: Vec<u8> = BagOfCellsIter::from_roots(&[root]).map(cell_name).collect();

        assert_eq!(order, vec![b'D', b'C', b'B', b'A', b'R']);
    }

    #[test]
    fn iter_dag_shared_cells_twice() {
        let root = make_dag();

        let order: Vec<u8> = BagOfCellsIter::from_roots(&[root]).map(cell_name).collect();

        assert_eq!(order, vec![b'C', b'C', b'B', b'A', b'R']);
    }

    #[test]
    fn augmented_tree_passes_children_values() {
        let root = make_tree();

        let result: Vec<(u8, u32)> = BagOfCellsIter::from_roots(&[root])
            .augmented(|_, children| children.sum::<u32>() + 1)
            .map(|(cell, sum)| (cell_name(cell), sum))
            .collect();

        assert_eq!(
            result,
            vec![(b'D', 1), (b'C', 1), (b'B', 2), (b'A', 2), (b'R', 5),]
        );
    }

    #[test]
    fn augmented_uniq_dag_each_cell_once_single_hashmap() {
        let root = make_dag();

        let result: Vec<(u8, u32)> = BagOfCellsIter::from_roots(&[root])
            .augmented(|_, children| children.sum::<u32>() + 1)
            .unique()
            .map(|(cell, sum)| (cell_name(cell), sum))
            .collect();

        assert_eq!(result, vec![(b'C', 1), (b'B', 2), (b'A', 2), (b'R', 5),]);
    }

    #[test]
    fn iter_multi_root_descendants_before_ancestors() {
        let c = make_cell(b'C', vec![]);
        let a = make_cell(b'A', vec![c.clone()]);
        let b = make_cell(b'B', vec![c]);
        let roots = vec![a, b];

        let order: Vec<u8> = BagOfCellsIter::from_roots(&roots).map(cell_name).collect();

        assert_eq!(order, vec![b'C', b'C', b'A', b'B']);
    }

    #[test]
    fn iter_multi_root_child_is_root() {
        let b = make_cell(b'B', vec![]);
        let a = make_cell(b'A', vec![b.clone()]);
        let roots = vec![a, b];

        let order: Vec<u8> = BagOfCellsIter::from_roots(&roots).map(cell_name).collect();

        assert_eq!(order, vec![b'B', b'A', b'B']);
    }
}
