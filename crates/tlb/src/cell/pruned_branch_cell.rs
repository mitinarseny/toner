use bitvec::order::Msb0;
use bitvec::prelude::BitVec;
use bytemuck::cast_slice;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct PrunedBranchCell {
    // TODO[akostylev0] maybe level == level_mask
    pub level: u8,
    pub data: BitVec<u8, Msb0>,
}

impl PrunedBranchCell {
    pub fn max_depth(&self) -> u16 {
        self.depths().iter().max().cloned().unwrap_or(0)
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

    fn depths(&self) -> &[u16] {
        let depths = &self.data.as_raw_slice()
            [(32 * self.level) as usize..(32 * self.level + 2 * self.level) as usize];

        cast_slice(depths)
    }

    pub fn hash(&self, idx: u8) -> Option<[u8; 32]> {
        if idx > self.level {
            return None;
        }

        Some(
            self.data.as_raw_slice()[(1 + 32 * idx) as usize..(1 + 32 * (idx + 1)) as usize]
                .try_into()
                .expect("invalid data length"),
        )
    }
}
