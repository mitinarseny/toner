use crate::level_mask::LevelMask;

pub trait HigherHash {
    fn level_mask(&self) -> LevelMask;

    // TODO[akostylev0]
    fn higher_hash(&self, level: u8) -> Option<[u8; 32]>;

    fn depth(&self, level: u8) -> u16;
}
