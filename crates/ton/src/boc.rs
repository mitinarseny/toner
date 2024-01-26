use std::sync::Arc;

use tlb::Cell;

type BoC = BagOfCells;

#[derive(Debug, Clone)]
pub struct BagOfCells {
    roots: Vec<Arc<Cell>>,
}

impl BagOfCells {
    pub fn from_root(root: impl Into<Arc<Cell>>) -> Self {
        Self {
            roots: [root.into()].into(),
        }
    }

    pub fn add_root(&mut self, root: impl Into<Arc<Cell>>) {
        self.roots.push(root.into())
    }

    pub fn single_root(&self) -> Option<&Arc<Cell>> {
        let [root]: &[_; 1] = self.roots.as_slice().try_into().ok()?;
        Some(root)
    }
}

#[cfg(feature = "tonlib")]
mod tonlib {
    use core::ops::Deref;

    use ::tonlib::cell::BagOfCells as TonlibBoC;

    use super::*;

    impl From<&TonlibBoC> for BoC {
        fn from(boc: &TonlibBoC) -> Self {
            Self {
                roots: boc
                    .roots
                    .iter()
                    .map(Deref::deref)
                    .map(Into::into)
                    .map(Arc::new)
                    .collect(),
            }
        }
    }

    impl From<&BoC> for TonlibBoC {
        fn from(boc: &BoC) -> Self {
            Self {
                roots: boc
                    .roots
                    .iter()
                    .map(Deref::deref)
                    .map(Into::into)
                    .map(Arc::new)
                    .collect(),
            }
        }
    }
}
