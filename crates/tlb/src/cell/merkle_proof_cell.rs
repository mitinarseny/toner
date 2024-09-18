use crate::cell::higher_hash::HigherHash;
use crate::Cell;
use bitvec::order::Msb0;
use bitvec::prelude::BitVec;
use std::cmp::max;
use std::sync::Arc;
use sha2::{Digest, Sha256};
use crate::cell_type::CellType;
use crate::level_mask::LevelMask;

#[derive(Clone, Default, PartialEq, Eq, Hash)]
pub struct MerkleProofCell {
    pub data: BitVec<u8, Msb0>,
    pub references: Vec<Arc<Cell>>,
}

impl HigherHash for MerkleProofCell {
    fn higher_hash(&self, level: u8) -> [u8; 32] {
        todo!()
        // debug_assert!(level <= 3);
        // if level > 3 || level > self.level() {
        //     return None;
        // }
        //
        // let mut buf = Vec::new();
        // buf.push(self.refs_descriptor());
        // buf.push(self.bits_descriptor());
        //
        // buf.push(CellType::MerkleProof as u8);
        // buf.extend(self.data.as_raw_slice());
        //
        // // ref depth
        // buf.extend(self.reference().max_depth().to_be_bytes());
        //
        // // ref hashes
        // buf.extend(self.reference().higher_hash(level + 1)?);
        //
        // let mut hasher = Sha256::new();
        // hasher.update(buf);
        //
        // Some(hasher.finalize().into())
    }

    fn level_mask(&self) -> LevelMask {
        self.reference().level_mask().shift(1)
    }

    fn depth(&self, level: u8) -> u16 {
        todo!()
    }
}

impl MerkleProofCell {
    pub fn hash(&self) -> [u8; 32] {
        self.data.as_raw_slice()[0..32]
            .try_into()
            .expect("invalid data length")
    }

    pub fn reference(&self) -> Arc<Cell> {
        self.references
            .first()
            .cloned()
            .expect("must have exactly one reference")
    }

    pub fn level(&self) -> u8 {
        max(self.reference().level() - 1, 0)
    }

    pub fn max_depth(&self) -> u16 {
        self.reference().max_depth() + 1
    }

    pub fn verify(&self) -> bool {
        debug_assert_eq!(self.hash(), self.reference().higher_hash(0));
        
        self.hash() == self.reference().higher_hash(0)
    }

    #[inline]
    fn refs_descriptor(&self) -> u8 {
        1 + 8 + 32 * self.level()
    }

    /// See [Cell serialization](https://docs.ton.org/develop/data-formats/cell-boc#cell-serialization)
    #[inline]
    const fn bits_descriptor(&self) -> u8 {
        let b = 280;

        (b / 8) as u8 + ((b + 7) / 8) as u8
    }
}
