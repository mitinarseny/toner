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

    pub fn from_root(root: impl Into<Arc<Cell>>) -> Self {
        Self::from_iter(iter::once(root))
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
    pub fn single_root(&self) -> Option<&Arc<Cell>> {
        let [root]: &[_; 1] = self.0.as_slice().try_into().ok()?;
        Some(root)
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
    use crate::{NBits, Ref, TLBSerializeExt, TLBSerializeWrapAs};

    #[test]
    fn boc() {
        let cell = (
            0b1.wrap_as::<NBits<1>>(),
            0x0AAAAA.wrap_as::<NBits<24>>().wrap_as::<Ref>(),
            (
                0x7E.wrap_as::<NBits<7>>(),
                0x0AAAAA.wrap_as::<NBits<24>>().wrap_as::<Ref>(),
            )
                .wrap_as::<Ref>(),
        )
            .to_cell()
            .unwrap();

        let boc = BoC::from_root(cell);

        assert_eq!(
            boc.serialize(),
            hex!("f345277cc6cfa747f001367e1e873dcfa8a936b8492431248b7a3eeafa8030e7")
        );
    }
}
