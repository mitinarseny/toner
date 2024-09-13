use bitvec::order::Msb0;
use bitvec::vec::BitVec;
use crate::cell::higher_hash::HigherHash;

#[derive(Clone, Default, PartialEq, Eq, Hash)]
pub struct LibraryReferenceCell {
    pub data: BitVec<u8, Msb0>,
}

impl LibraryReferenceCell {
    pub fn hash(&self) -> [u8; 32] {
        self.data.as_raw_slice().try_into().expect("invalid hash length")
    }
}

impl HigherHash for LibraryReferenceCell {
    fn higher_hash(&self, level: u8) -> Option<[u8; 32]> {
        if level == 0 {
            Some(self.hash())
        } else {
            None
        }
    }
}
