#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExoticCellKind {
    PrunedBranch,
    LibraryReference,
    MerkleProof,
    MerkleUpdate,
    Unknown { tag: u8 },
}

impl ExoticCellKind {
    #[inline]
    pub const fn is_pruned_branch(&self) -> bool {
        matches!(self, Self::PrunedBranch)
    }

    #[inline]
    pub const fn is_merkle(&self) -> bool {
        matches!(self, Self::MerkleProof | Self::MerkleUpdate)
    }
}
