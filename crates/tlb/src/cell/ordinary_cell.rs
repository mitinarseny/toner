use crate::cell::higher_hash::HigherHash;
use crate::Cell;
use bitvec::order::Msb0;
use bitvec::prelude::BitVec;
use sha2::{Digest, Sha256};
use std::ops::Deref;
use std::sync::Arc;

#[derive(Clone, Default, PartialEq, Eq, Hash)]
pub struct OrdinaryCell {
    pub data: BitVec<u8, Msb0>,
    pub references: Vec<Arc<Cell>>,
}

impl HigherHash for OrdinaryCell {
    /// [Standard Cell representation hash](https://docs.ton.org/develop/data-formats/cell-boc#standard-cell-representation-hash-calculation)
    fn higher_hash(&self, level: u8) -> Option<[u8; 32]> {
        debug_assert!(level <= 3);
        if level > 3 {
            return None;
        }

        let mut buf = Vec::new();
        buf.push(self.refs_descriptor());
        buf.push(self.bits_descriptor());

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

        // refs depth
        buf.extend(
            self.references
                .iter()
                .flat_map(|r| r.max_depth().to_be_bytes()),
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

        let mut hasher = Sha256::new();
        hasher.update(buf);

        Some(hasher.finalize().into())
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
    fn refs_descriptor(&self) -> u8 {
        self.references.len() as u8 | (self.level() << 5)
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
