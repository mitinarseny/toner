use bitvec::order::Msb0;
use bitvec::vec::BitVec;

#[derive(Clone, Default, PartialEq, Eq, Hash)]
pub struct LibraryReferenceCell {
    pub data: BitVec<u8, Msb0>,
}

impl LibraryReferenceCell {
    pub fn hash(&self) -> [u8; 32] {
        self.data.as_raw_slice().try_into().expect("invalid hash length")
    }
}
