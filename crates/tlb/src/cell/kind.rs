#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CellKind {
    #[default]
    Ordinary,
    PrunedBranch,
    LibraryReference,
    MerkleProof,
    MerkleUpdate,
}

impl CellKind {
    #[inline]
    pub fn is_pruned_branch(&self) -> bool {
        matches!(self, CellKind::PrunedBranch)
    }

    #[inline]
    pub fn is_merkle(&self) -> bool {
        matches!(self, CellKind::MerkleProof | CellKind::MerkleUpdate)
    }
}
