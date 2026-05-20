#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LevelMask(u8);

impl LevelMask {
    #[inline]
    pub const fn new(mask: u8) -> Self {
        Self(mask)
    }

    #[inline]
    pub const fn value(self) -> u8 {
        self.0
    }

    #[inline]
    pub const fn level(self) -> u8 {
        (8 - self.0.leading_zeros()) as u8
    }

    #[inline]
    pub const fn hash_index(self) -> usize {
        self.0.count_ones() as usize
    }

    #[inline]
    pub fn limited_by(self, level: u8) -> Self {
        Self(self.0 & ((1u8 << level).wrapping_sub(1)))
    }

    #[inline]
    pub const fn contains(self, level: u8) -> bool {
        level == 0 || (self.0 >> (level.saturating_sub(1))) & 1 != 0
    }

    #[inline]
    pub const fn merkle_shift(self) -> Self {
        Self(self.0 >> 1)
    }
}

impl core::ops::BitOr for LevelMask {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}
