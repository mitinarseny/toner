use core::mem;

use bitvec::{
    mem::bits_of,
    order::Msb0,
    view::{AsBits, AsMutBits},
};
use impl_tools::autoimpl;

use crate::{CellBuilder, CellParser, Error, Result, TLBDeserialize, TLBSerialize};

#[autoimpl(Deref using self.0)]
#[autoimpl(DerefMut using self.0)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Num<const BITS: u32, T>(pub T);

impl<const BITS: u32, T> Num<BITS, T> {
    const BITS_SIZE: u32 = bits_of::<T>() as u32;

    #[inline]
    pub const fn new(v: T) -> Self {
        assert!(BITS <= Self::BITS_SIZE, "excessive bits for type");
        Self(v)
    }
}

impl<const BITS: u32, T> From<T> for Num<BITS, T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

macro_rules! impl_tlb_serialize_for_integers {
    ($($t:tt)+) => {$(
        impl TLBSerialize for $t {
            #[inline]
            fn store(&self, builder: &mut CellBuilder) -> Result<()> {
                self.to_be_bytes().as_bits::<Msb0>().store(builder)
            }
        }

        impl TLBDeserialize for $t {
            #[inline]
            fn parse(parser: &mut CellParser<'_>) -> Result<Self> {
                Ok(Self::from_be_bytes(parser.load_bytes_array()?))
            }
        }

        impl<const BITS: u32> TLBSerialize for Num<BITS, $t> {
            fn store(&self, builder: &mut CellBuilder) -> Result<()> {
                assert!(BITS <= Self::BITS_SIZE, "excessive bits for type");
                if BITS < Self::BITS_SIZE - self.leading_zeros() {
                    return Err(Error::TooShort);
                }
                let bytes = self.to_be_bytes();
                let mut bits = bytes.as_bits::<Msb0>();
                bits = &bits[bits.len() - BITS as usize..];
                builder.store(bits)?;
                Ok(())
            }
        }

        impl<const BITS: u32> TLBDeserialize for Num<BITS, $t> {
            fn parse(parser: &mut CellParser<'_>) -> Result<Self> {
                let bits = parser.load_bits(BITS as usize)?;
                let mut arr = [0u8; mem::size_of::<$t>()];
                arr.as_mut_bits()[(Self::BITS_SIZE - BITS) as usize..]
                    .copy_from_bitslice(bits);
                Ok(Self::new($t::from_be_bytes(arr)))
            }
        }
    )+};
}
impl_tlb_serialize_for_integers! {
    u8 u16 u32 u64 u128
    i8 i16 i32 i64 i128
}

#[cfg(test)]
mod tests {
    use bitvec::{bits, order::Msb0};

    use crate::{Cell, TLBDeserializeExt, TLBSerializeExt};

    use super::*;

    #[test]
    fn uint_serialize() {
        assert_eq!(
            0xFD_FE_u16.to_cell().unwrap(),
            Cell::builder()
                .with(bits![u8, Msb0; 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0])
                .unwrap()
                .into_cell(),
        )
    }

    #[test]
    fn uint_serde() {
        const N: u32 = 12345;
        assert_eq!(u32::parse_fully(&N.to_cell().unwrap()).unwrap(), N)
    }

    #[test]
    fn num_tlb_serialize() {
        assert_eq!(
            Num::<7, u8>(0x7E).to_cell().unwrap(),
            Cell::builder()
                .with(bits![u8, Msb0; 1, 1, 1, 1, 1, 1, 0])
                .unwrap()
                .into_cell(),
        )
    }

    #[test]
    fn num_one_bit() {
        assert_eq!(
            Num::<1, u8>(0b1).to_cell().unwrap(),
            true.to_cell().unwrap()
        )
    }

    #[test]
    fn num_same_uint() {
        assert_eq!(
            Num::<8, u8>(231).to_cell().unwrap(),
            231_u8.to_cell().unwrap()
        )
    }

    #[test]
    fn num_serde() {
        type T = Num<7, u8>;
        const N: u8 = 0x7E;

        assert_eq!(T::parse_fully(&T::new(N).to_cell().unwrap()).unwrap().0, N)
    }
}
