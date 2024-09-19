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
    fn level_mask(&self) -> LevelMask {
        self.reference().level_mask().shift(1)
    }

    fn higher_hash(&self, level: u8) -> [u8; 32] {
        let level_mask = self.level_mask();
        let max_level = level_mask.apply(level).as_level();

        (0..=max_level).fold(None, |acc, current_level| {
            let level_mask = level_mask.apply(current_level);
            let level = level_mask.as_level();

            let mut hasher = Sha256::new();
            hasher.update([self.refs_descriptor(), self.bits_descriptor()]);
            if let Some(prev) = acc {
                hasher.update(prev);
            } else {
                hasher.update([CellType::MerkleProof as u8]);
                let rest_bits = self.data.len() % 8;
                if rest_bits == 0 {
                    hasher.update(self.data.as_raw_slice());
                } else {
                    let (last, data) = self.data.as_raw_slice().split_last().unwrap();
                    hasher.update(data);
                    let mut last = last & (0xFF << (8 - rest_bits)); // clear the rest
                    last |= 1 << (8 - rest_bits - 1); // put stop-bit
                    hasher.update([last])
                }
            }

            hasher.update(self.reference().depth(level + 1).to_be_bytes());
            hasher.update(self.reference().higher_hash(level + 1));

            Some(hasher.finalize().into())
        }).expect("level 0 is always present")
    }

    fn depth(&self, level: u8) -> u16 {
        self.reference().depth(level + 1) + 1
    }
}

impl MerkleProofCell {
    pub fn level(&self) -> u8 {
        max(self.reference().level() - 1, 0)
    }

    pub fn max_depth(&self) -> u16 {
        self.reference().max_depth() + 1
    }

    pub fn verify(&self) -> bool {
        self.data.as_raw_slice()[0..32] == self.reference().higher_hash(0)
    }

    fn reference(&self) -> Arc<Cell> {
        self.references
            .first()
            .cloned()
            .expect("must have exactly one reference")
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
