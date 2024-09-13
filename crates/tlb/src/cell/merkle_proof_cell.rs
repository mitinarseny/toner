use crate::Cell;
use bitvec::order::Msb0;
use bitvec::prelude::BitVec;
use std::cmp::max;
use std::sync::Arc;

#[derive(Clone, Default, PartialEq, Eq, Hash)]
pub struct MerkleProofCell {
    pub level: u8,
    pub data: BitVec<u8, Msb0>,
    pub references: Vec<Arc<Cell>>,
}

impl MerkleProofCell {
    pub fn hash(&self) -> [u8; 32] {
        self.data.as_raw_slice()[0..32]
            .try_into()
            .expect("invalid data length")
    }

    pub fn level(&self) -> u8 {
        let level = max(
            self.references
                .first()
                .expect("must have exactly one reference")
                .level()
                - 1,
            0,
        );

        // TODO[akostylev0]
        debug_assert_eq!(self.level, level);

        level
    }

    pub fn max_depth(&self) -> u16 {
        todo!()
    }

    pub fn verify(&self) -> bool {
        self.hash()
            == self
                .references
                .first()
                .expect("must have exactly one reference")
                .hash()
    }
}
