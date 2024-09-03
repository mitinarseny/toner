use std::fmt::{Debug, Display, Formatter};

#[derive(Clone, PartialEq, Eq, Hash, Copy, Debug)]
pub enum CellType {
    Ordinary,
    PrunedBranch,
    LibraryReference,
    MerkleProof,
    MerkleUpdate,
}

impl Display for CellType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl Default for CellType {
    fn default() -> Self {
        Self::Ordinary
    }
}
