#[derive(Clone, PartialEq, Eq, Hash, Copy)]
pub enum CellType {
    Ordinary,
    PrunedBranch,
    LibraryReference,
    MerkleProof,
    MerkleUpdate,
}

impl Default for CellType {
    fn default() -> Self {
        Self::Ordinary
    }
}
