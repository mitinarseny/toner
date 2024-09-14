use crate::cell::higher_hash::HigherHash;
use crate::level_mask::LevelMask;
use crate::Cell;
use bitvec::order::Msb0;
use bitvec::prelude::BitVec;
use sha2::{Digest, Sha256};
use std::ops::{BitOr, Deref};
use std::sync::Arc;

#[derive(Clone, Default, PartialEq, Eq, Hash)]
pub struct OrdinaryCell {
    pub data: BitVec<u8, Msb0>,
    pub references: Vec<Arc<Cell>>,
}

impl HigherHash for OrdinaryCell {
    fn level_mask(&self) -> LevelMask {
        self.references
            .iter()
            .map(Deref::deref)
            .map(Cell::level_mask)
            .fold(LevelMask::default(), LevelMask::bitor)
    }

    /// [Standard Cell representation hash](https://docs.ton.org/develop/data-formats/cell-boc#standard-cell-representation-hash-calculation)
    fn higher_hash(&self, level: u8) -> Option<[u8; 32]> {
        let level_mask = self.level_mask().apply(level);
        let level = level_mask.as_level();

        let mut buf = Vec::new();
        buf.push(self.refs_descriptor(level_mask));
        buf.push(self.bits_descriptor());

        println!("level = {}, level_mask = {:?}", level, self.level_mask());
        if level > 0 {
            buf.extend(self.higher_hash(level - 1)?);
        } else {
            let rest_bits = self.data.len() % 8;

            if rest_bits == 0 {
                buf.extend(self.data.as_raw_slice());
            } else {
                let (last, data) = self.data.as_raw_slice().split_last().unwrap();
                buf.extend(data);
                let mut last = last & (!0u8 << (8 - rest_bits)); // clear the rest
                // let mut last = last;
                last |= 1 << (8 - rest_bits - 1); // put stop-bit
                buf.push(last)
            }
        }

        // refs depth
        buf.extend(
            self.references
                .iter()
                .flat_map(|r| r.depth(level).to_be_bytes()),
        );

        // refs hashes
        buf.extend(
            self.references
                .iter()
                .map(|cell| cell.higher_hash(level))
                .collect::<Option<Vec<[u8; 32]>>>()?
                .iter()
                .flatten(),
        );

        println!("cell_data = {}", hex::encode(&buf));

        let mut hasher = Sha256::new();
        hasher.update(buf);

        Some(hasher.finalize().into())
    }

    fn depth(&self, level: u8) -> u16 {
        self.references
            .iter()
            .map(Deref::deref)
            .map(|c| c.depth(level))
            .max()
            .map(|v| v + 1)
            .unwrap_or(0)
    }
}

impl OrdinaryCell {
    #[inline]
    pub fn hash(&self) -> [u8; 32] {
        self.higher_hash(0).expect("level 0 is always present")
    }

    #[inline]
    pub fn level(&self) -> u8 {
        self.references
            .iter()
            .map(Deref::deref)
            .map(Cell::level)
            .max()
            .unwrap_or(0)
    }

    #[inline]
    fn refs_descriptor(&self, level_mask: LevelMask) -> u8 {
        self.references.len() as u8 | (level_mask.as_u8() << 5)
    }

    /// See [Cell serialization](https://docs.ton.org/develop/data-formats/cell-boc#cell-serialization)
    #[inline]
    fn bits_descriptor(&self) -> u8 {
        let b = self.data.len();

        (b / 8) as u8 + ((b + 7) / 8) as u8
    }
}

#[cfg(test)]
mod tests {
    use crate::cell::higher_hash::HigherHash;
    use crate::OrdinaryCell;

    #[test]
    fn ordinary_cell_higher_hash_equals_if_no_refs() {
        let cell = OrdinaryCell::default();

        assert_eq!(cell.higher_hash(0), cell.higher_hash(1));
        assert_eq!(cell.higher_hash(1), cell.higher_hash(2));
        assert_eq!(cell.higher_hash(2), cell.higher_hash(3));
    }
}
