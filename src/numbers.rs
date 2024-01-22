use core::mem;

use bitvec::{
    mem::bits_of,
    order::Msb0,
    view::{AsBits, AsMutBits},
};
use num_bigint::{BigInt, BigUint};

use crate::{
    CellBuilder, CellParser, ErrorReason, NBits, Result, TLBDeserialize, TLBDeserializeAs,
    TLBSerialize, TLBSerializeAs, TLBSerializeWrapAs, VarBytes,
};

pub struct ConstBit<const VALUE: bool>;

impl<const VALUE: bool> TLBSerialize for ConstBit<VALUE> {
    fn store(&self, builder: &mut CellBuilder) -> Result<()> {
        VALUE.store(builder)
    }
}

impl<'de, const VALUE: bool> TLBDeserialize<'de> for ConstBit<VALUE> {
    fn parse(parser: &mut CellParser<'de>) -> Result<Self> {
        if VALUE != parser.parse::<bool>()? {
            Err(ErrorReason::custom(format!(
                "expected {:#b}, got {:#b}",
                VALUE as u8, !VALUE as u8
            ))
            .into())
        } else {
            Ok(Self)
        }
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

        impl<'de> TLBDeserialize<'de> for $t {
            #[inline]
            fn parse(parser: &mut CellParser<'de>) -> Result<Self> {
                const BYTES_SIZE: usize = mem::size_of::<$t>();
                let bits = parser.load_bytes(BYTES_SIZE)?;
                let mut arr = [0u8; BYTES_SIZE];
                arr.as_mut_bits().copy_from_bitslice(bits);
                Ok(Self::from_be_bytes(arr))
            }
        }

        impl<const BITS: usize> TLBSerializeAs<$t> for NBits<BITS> {
            fn store_as(source: &$t, builder: &mut CellBuilder) -> Result<()> {
                const BITS_SIZE: usize = bits_of::<$t>();
                assert!(BITS <= BITS_SIZE, "excessive bits for type");
                if BITS < BITS_SIZE - source.leading_zeros() as usize {
                    return Err(ErrorReason::TooShort.into());
                }
                let bytes = source.to_be_bytes();
                let mut bits = bytes.as_bits::<Msb0>();
                bits = &bits[bits.len() - BITS..];
                builder.store(bits)?;
                Ok(())
            }
        }

        impl<'de, const BITS: usize> TLBDeserializeAs<'de, $t> for NBits<BITS> {
            fn parse_as(parser: &mut CellParser<'de>) -> Result<$t> {
                const BITS_SIZE: usize = bits_of::<$t>();
                assert!(BITS <= BITS_SIZE, "excessive bits for type");
                let bits = Self::load(parser)?;
                let mut arr = [0u8; mem::size_of::<$t>()];
                arr.as_mut_bits()[BITS_SIZE - BITS..]
                    .copy_from_bitslice(bits);
                Ok($t::from_be_bytes(arr))
            }
        }
    )+};
}
impl_tlb_serialize_for_integers! {
    u8 u16 u32 u64 u128 usize
    i8 i16 i32 i64 i128 isize
}

pub struct ConstUint<const VALUE: u32, const BITS: usize>;

impl<const VALUE: u32, const BITS: usize> TLBSerialize for ConstUint<VALUE, BITS> {
    fn store(&self, builder: &mut CellBuilder) -> Result<()> {
        VALUE.wrap_as::<NBits<BITS>>().store(builder)
    }
}

impl<'de, const VALUE: u32, const BITS: usize> TLBDeserialize<'de> for ConstUint<VALUE, BITS> {
    fn parse(parser: &mut CellParser<'de>) -> Result<Self> {
        let v = parser.parse_as::<u32, NBits<BITS>>()?;
        if v != VALUE {
            return Err(
                ErrorReason::custom(format!("expected const {VALUE:#b}, got: {v:#b}")).into(),
            );
        }
        Ok(Self)
    }
}

impl<const BITS: usize> TLBSerializeAs<BigUint> for NBits<BITS> {
    fn store_as(source: &BigUint, builder: &mut CellBuilder) -> Result<()> {
        let used_bits = source.bits() as usize;
        if BITS < used_bits {
            return Err(ErrorReason::TooShort.into());
        }

        builder.repeat_bit(BITS - used_bits, false)?;

        let bytes = source.to_bytes_be();
        let mut bits = bytes.as_bits::<Msb0>();
        bits = &bits[bits.len() - used_bits..];
        builder.store(bits)?;
        Ok(())
    }
}

impl<'de, const BITS: usize> TLBDeserializeAs<'de, BigUint> for NBits<BITS> {
    fn parse_as(parser: &mut CellParser<'de>) -> Result<BigUint> {
        let mut bits = Self::load(parser)?.to_bitvec();
        let total_bits = (BITS + 7) & !7;
        bits.resize(total_bits, false);
        bits.shift_right(total_bits - BITS);
        Ok(BigUint::from_bytes_be(bits.as_raw_slice()))
    }
}

pub struct VarUint<const BITS_FOR_BYTES_LEN: usize>;

impl<const BITS_FOR_BYTES_LEN: usize> TLBSerializeAs<BigUint> for VarUint<BITS_FOR_BYTES_LEN> {
    #[inline]
    fn store_as(source: &BigUint, builder: &mut CellBuilder) -> Result<()> {
        VarBytes::<BITS_FOR_BYTES_LEN>::store_as(&source.to_bytes_be(), builder)
    }
}

impl<'de, const BITS_FOR_BYTES_LEN: usize> TLBDeserializeAs<'de, BigUint>
    for VarUint<BITS_FOR_BYTES_LEN>
{
    fn parse_as(parser: &mut CellParser<'de>) -> Result<BigUint> {
        let mut bits = parser
            .parse_as::<_, VarBytes<BITS_FOR_BYTES_LEN>>()?
            .to_bitvec();
        let total_bits = (bits.len() + 7) & !7;
        let shift = total_bits - bits.len();
        bits.resize(total_bits, false);
        bits.shift_right(shift);
        Ok(BigUint::from_bytes_be(bits.as_raw_slice()))
    }
}

pub struct VarInt<const BITS_FOR_BYTES_LEN: usize>;

impl<const BITS_FOR_BYTES_LEN: usize> TLBSerializeAs<BigInt> for VarInt<BITS_FOR_BYTES_LEN> {
    #[inline]
    fn store_as(source: &BigInt, builder: &mut CellBuilder) -> Result<()> {
        VarBytes::<BITS_FOR_BYTES_LEN>::store_as(&source.to_signed_bytes_be(), builder)
    }
}

impl<'de, const BITS_FOR_BYTES_LEN: usize> TLBDeserializeAs<'de, BigInt>
    for VarInt<BITS_FOR_BYTES_LEN>
{
    fn parse_as(parser: &mut CellParser<'de>) -> Result<BigInt> {
        let mut bits = parser
            .parse_as::<_, VarBytes<BITS_FOR_BYTES_LEN>>()?
            .to_bitvec();
        let total_bits = (bits.len() + 7) & !7;
        let shift = total_bits - bits.len();
        bits.resize(total_bits, false);
        bits.shift_right(shift);
        Ok(BigInt::from_signed_bytes_be(bits.as_raw_slice()))
    }
}

pub type Coins = VarUint<4>;

#[cfg(test)]
mod tests {
    use bitvec::{bits, order::Msb0};

    use crate::{Cell, TLBDeserializeExt, TLBSerializeExt, TLBSerializeWrapAs};

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
    fn nbits_number_tlb_serialize() {
        assert_eq!(
            0x7E.wrap_as::<NBits<7>>().to_cell().unwrap(),
            Cell::builder()
                .with(bits![u8, Msb0; 1, 1, 1, 1, 1, 1, 0])
                .unwrap()
                .into_cell(),
        )
    }

    #[test]
    fn nbits_one_bit() {
        assert_eq!(
            0b1.wrap_as::<NBits<1>>().to_cell().unwrap(),
            true.to_cell().unwrap()
        )
    }

    #[test]
    fn nbits_same_uint() {
        assert_eq!(
            231.wrap_as::<NBits<8>>().to_cell().unwrap(),
            231_u8.to_cell().unwrap()
        )
    }

    #[test]
    fn nbits_number_serde() {
        type T = u8;
        const BITS: usize = 7;
        const N: T = 0x7E;

        assert_eq!(
            N.wrap_as::<NBits<BITS>>()
                .to_cell()
                .unwrap()
                .parser()
                .parse_fully_as::<T, NBits<BITS>>()
                .unwrap(),
            N
        )
    }

    #[test]
    fn big_nbits_serde() {
        const BITS: usize = 100;
        let n: BigUint = 12345_u64.into();
        assert_eq!(
            n.wrap_as::<NBits<BITS>>()
                .to_cell()
                .unwrap()
                .parser()
                .parse_fully_as::<BigUint, NBits<BITS>>()
                .unwrap(),
            n
        );
    }
}
