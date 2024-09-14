use crate::cell::higher_hash::HigherHash;
use crate::cell_type::CellType;
use bitvec::order::Msb0;
use bitvec::prelude::BitVec;
use sha2::{Digest, Sha256};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct PrunedBranchCell {
    // TODO[akostylev0] maybe level == level_mask
    pub level: u8,
    pub data: BitVec<u8, Msb0>,
}

impl HigherHash for PrunedBranchCell {
    fn higher_hash(&self, level: u8) -> Option<[u8; 32]> {
        if level == 0 {
            // TODO[akostylev0]: rly?

            let mut buf = Vec::new();
            buf.push(self.refs_descriptor());
            buf.push(self.bits_descriptor());

            buf.push(CellType::PrunedBranch as u8);
            buf.extend(self.data.as_raw_slice());

            // refs depth
            buf.extend(self.max_depth().to_be_bytes());

            let mut hasher = Sha256::new();
            hasher.update(buf);

            return Some(hasher.finalize().into());
        }

        Some(
            self.data.as_raw_slice()[1 + (32 * (level - 1)) as usize..1 + (32 * level) as usize]
                .try_into()
                .expect("invalid data length"),
        )
    }
}

impl PrunedBranchCell {
    pub fn max_depth(&self) -> u16 {
        self.depths().into_iter().max().unwrap_or(0)
    }

    pub fn level(&self) -> u8 {
        // TODO[akostylev0]
        debug_assert_eq!(self.level_mask(), self.level);

        self.level
    }

    pub fn level_mask(&self) -> u8 {
        self.data
            .as_raw_slice()
            .first()
            .cloned()
            .expect("invalid data length")
    }

    fn depths(&self) -> Vec<u16> {
        let depths = &self.data.as_raw_slice()
            [(1 + 32 * self.level) as usize..(1 + 32 * self.level + 2 * self.level) as usize];

        depths
            .chunks_exact(2)
            .map(|c| u16::from_be_bytes(c.try_into().unwrap()))
            .collect()
    }

    #[inline]
    fn refs_descriptor(&self) -> u8 {
        0 + 8 + 32 * self.level()
    }

    /// See [Cell serialization](https://docs.ton.org/develop/data-formats/cell-boc#cell-serialization)
    #[inline]
    fn bits_descriptor(&self) -> u8 {
        let b = self.data.len() + 8;

        (b / 8) as u8 + ((b + 7) / 8) as u8
    }
}
