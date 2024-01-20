use core::{fmt::Debug, iter};
use std::sync::Arc;

use impl_tools::autoimpl;

use crate::Cell;

pub type BoC = BagOfCells;

#[autoimpl(Deref using self.0)]
#[autoimpl(AsRef using self.0)]
#[derive(Clone)]
pub struct BagOfCells(Vec<Arc<Cell>>);

impl BagOfCells {
    #[inline]
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    #[inline]
    pub fn roots(&self) -> &[Arc<Cell>] {
        &self.0
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.roots().len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn single_root(root: impl Into<Arc<Cell>>) -> Self {
        Self::from_iter(iter::once(root))
    }

    #[inline]
    pub fn push_root(&mut self, root: impl Into<Arc<Cell>>) {
        self.0.push(root.into())
    }

    pub fn serialize(&self) -> Vec<u8> {
        todo!()
    }
}

impl Default for BagOfCells {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<A> FromIterator<A> for BagOfCells
where
    A: Into<Arc<Cell>>,
{
    #[inline]
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
        Self(iter.into_iter().map(Into::into).collect())
    }
}

impl<A> Extend<A> for BagOfCells
where
    A: Into<Arc<Cell>>,
{
    #[inline]
    fn extend<T: IntoIterator<Item = A>>(&mut self, iter: T) {
        self.0.extend(iter.into_iter().map(Into::into))
    }
}

impl Debug for BagOfCells {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_set().entries(&self.0).finish()
    }
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use super::*;
    use crate::{Num, Ref, TLBSerializeExt};

    #[test]
    fn boc() {
        let cell = (
            Num::<1, u8>(0b1),
            Ref(Num::<24, u32>(0x0AAAAA)),
            Ref((Num::<7, u8>(0x7E), Ref(Num::<24, u32>(0x0AAAAA)))),
        )
            .to_cell()
            .unwrap();

        let boc = BoC::single_root(cell);

        assert_eq!(
            boc.serialize(),
            hex!("f345277cc6cfa747f001367e1e873dcfa8a936b8492431248b7a3eeafa8030e7")
        );
    }
}
