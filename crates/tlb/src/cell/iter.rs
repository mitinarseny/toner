use std::ops::Deref;

use crate::Cell;

#[derive(Debug, Clone)]
pub(crate) enum Frame<'a> {
    Visit(&'a Cell),
    Emit(&'a Cell),
}

/// DFS post-order iterator over a Cell DAG.
///
/// Each cell is yielded **after all of its descendants**, with children
/// visited left-to-right. For `R → [A, B], A → [C], B → [D]` this yields
/// `C, A, D, B, R`.
#[derive(Debug, Clone)]
pub struct CellIter<'a> {
    stack: Vec<Frame<'a>>,
}

impl<'a> CellIter<'a> {
    #[inline]
    pub fn new(root: &'a Cell) -> Self {
        Self {
            stack: vec![Frame::Visit(root)],
        }
    }

    #[inline]
    pub fn augmented<A, F>(self, f: F) -> AugmentedCellIter<'a, F, A>
    where
        F: FnMut(&'a Cell, &[A]) -> A,
        A: Clone,
    {
        AugmentedCellIter {
            inner: self,
            values: Vec::new(),
            f,
        }
    }
}

impl<'a> Iterator for CellIter<'a> {
    type Item = &'a Cell;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(frame) = self.stack.pop() {
            match frame {
                Frame::Visit(cell) => {
                    self.stack.push(Frame::Emit(cell));
                    self.stack.extend(
                        cell.references
                            .iter()
                            .rev()
                            .map(Deref::deref)
                            .map(Frame::Visit),
                    );
                }
                Frame::Emit(cell) => return Some(cell),
            }
        }

        None
    }
}

#[derive(Debug, Clone)]
pub struct AugmentedCellIter<'a, F, A> {
    inner: CellIter<'a>,
    values: Vec<A>,
    f: F,
}

impl<'a, F, A> Iterator for AugmentedCellIter<'a, F, A>
where
    F: FnMut(&'a Cell, &[A]) -> A,
    A: Clone,
{
    type Item = (&'a Cell, A);

    fn next(&mut self) -> Option<Self::Item> {
        let cell = self.inner.next()?;
        let split = self.values.len() - cell.references.len();
        let value = (self.f)(cell, &self.values[split..]);
        self.values.truncate(split);
        self.values.push(value.clone());
        Some((cell, value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::tests::*;

    #[test]
    fn iter_tree_post_order_left_to_right() {
        let root = make_tree();

        let order: Vec<u8> = CellIter::new(&root).map(cell_name).collect();

        assert_eq!(order, vec![b'C', b'A', b'D', b'B', b'R']);
    }

    #[test]
    fn iter_dag_shared_cells_twice() {
        let root = make_dag();

        let order: Vec<u8> = CellIter::new(&root).map(cell_name).collect();

        assert_eq!(order, vec![b'C', b'A', b'C', b'B', b'R']);
    }

    #[test]
    fn augmented_tree_passes_children_values() {
        let root = make_tree();

        let result: Vec<(u8, u32)> = CellIter::new(&root)
            .augmented(|_, children: &[u32]| children.iter().sum::<u32>() + 1)
            .map(|(cell, sum)| (cell_name(cell), sum))
            .collect();

        assert_eq!(
            result,
            vec![(b'C', 1), (b'A', 2), (b'D', 1), (b'B', 2), (b'R', 5),]
        );
    }

    #[test]
    fn augmented_dag_no_memoization() {
        let root = make_dag();

        let result: Vec<(u8, u32)> = CellIter::new(&root)
            .augmented(|_, children: &[u32]| children.iter().sum::<u32>() + 1)
            .map(|(cell, sum)| (cell_name(cell), sum))
            .collect();

        assert_eq!(
            result,
            vec![(b'C', 1), (b'A', 2), (b'C', 1), (b'B', 2), (b'R', 5),]
        );
    }
}
