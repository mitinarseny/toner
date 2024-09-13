pub trait HigherHash {
    // TODO[akostylev0]
    fn higher_hash(&self, level: u8) -> Option<[u8; 32]>;
}
