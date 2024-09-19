use crate::cell_type::CellType;
use crate::higher_hash::HigherHash;
use crate::level_mask::LevelMask;
use crate::Cell;
use bitvec::order::Msb0;
use bitvec::prelude::BitVec;
use sha2::{Digest, Sha256};
use std::cmp::max;
use std::ops::{BitOr, Deref};
use std::sync::Arc;

#[derive(Clone, Default, PartialEq, Eq, Hash)]
pub struct MerkleUpdateCell {
    pub data: BitVec<u8, Msb0>,
    pub references: Vec<Arc<Cell>>,
}

impl HigherHash for MerkleUpdateCell {
    fn level_mask(&self) -> LevelMask {
        self.references
            .iter()
            .map(Deref::deref)
            .map(HigherHash::level_mask)
            .reduce(BitOr::bitor)
            .map(|l| l.shift(1))
            .expect("merkle update cell must have exactly two references")
    }

    fn higher_hash(&self, level: u8) -> [u8; 32] {
        let level_mask = self.level_mask();
        let max_level = level_mask.apply(level).as_level();

        (0..=max_level)
            .fold(None, |acc, current_level| {
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

                // refs depth
                self.references
                    .iter()
                    .map(|r| r.depth(level).to_be_bytes())
                    .for_each(|bytes| hasher.update(bytes));

                // refs hashes
                self.references
                    .iter()
                    .map(|cell| cell.higher_hash(level))
                    .for_each(|bytes| hasher.update(bytes));

                Some(hasher.finalize().into())
            })
            .expect("level 0 is always present")
    }

    fn depth(&self, level: u8) -> u16 {
        self.references
            .iter()
            .map(Deref::deref)
            .map(|c| c.depth(level + 1))
            .max()
            .map(|v| v + 1)
            .unwrap_or(0)
    }
}

impl MerkleUpdateCell {
    pub fn level(&self) -> u8 {
        max(
            self.references
                .iter()
                .map(|r| r.level() - 1)
                .max()
                .unwrap_or(0),
            0,
        )
    }

    pub fn max_depth(&self) -> u16 {
        self.references
            .iter()
            .map(Deref::deref)
            .map(Cell::max_depth)
            .max()
            .map(|d| d + 1)
            .unwrap_or(0)
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
