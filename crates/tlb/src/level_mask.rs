use std::ops::{BitAnd, BitOr};

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct LevelMask(u8);

impl LevelMask {
    pub fn new(mask: u8) -> Self {
        Self(mask)
    }

    pub fn from_level(level: u8) -> Self {
        Self((1 << level) - 1)
    }

    pub fn as_level(&self) -> u8 {
        self.0.count_ones() as u8
    }

    pub fn shift(&self, amount: u8) -> LevelMask {
        Self(self.0 >> amount)
    }

    pub fn as_u8(&self) -> u8 {
        self.0
    }

    pub fn contains(&self, level: u8) -> bool {
        level < self.as_level()
    }

    pub fn apply(&self, level: u8) -> LevelMask {
        LevelMask(self.0 & ((1 << level) - 1))
    }
}

impl BitOr for LevelMask {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitAnd for LevelMask {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}
