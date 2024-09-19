use crate::cell::higher_hash::HigherHash;
use crate::level_mask::LevelMask;
use bitvec::order::Msb0;
use bitvec::vec::BitVec;

#[derive(Clone, Default, PartialEq, Eq, Hash)]
pub struct LibraryReferenceCell {
    pub data: BitVec<u8, Msb0>,
}

impl LibraryReferenceCell {
    #[inline]
    pub fn max_depth(&self) -> u16 {
        0
    }

    #[inline]
    pub fn level(&self) -> u8 {
        0
    }
    #[inline]
    pub fn hash(&self) -> [u8; 32] {
        self.data
            .as_raw_slice()
            .try_into()
            .expect("invalid hash length")
    }
}

impl HigherHash for LibraryReferenceCell {
    fn level_mask(&self) -> LevelMask {
        LevelMask::default()
    }

    fn higher_hash(&self, _: u8) -> [u8; 32] {
        self.hash()
    }

    fn depth(&self, _: u8) -> u16 {
        0
    }
}
