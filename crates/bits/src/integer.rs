use core::mem;

use bitvec::{
    mem::bits_of,
    order::Msb0,
    view::{AsBits, AsMutBits},
};

use crate::{
    AsBytes, BitPack, BitPackAs, BitReader, BitReaderExt, BitUnpack, BitUnpackAs, BitWriter,
    BitWriterExt, Error, NBits,
};

pub struct ConstBit<const VALUE: bool>;

impl<const VALUE: bool> BitPack for ConstBit<VALUE> {
    #[inline]
    fn pack<W>(&self, writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        VALUE.pack(writer)
    }
}

impl<const VALUE: bool> BitUnpack for ConstBit<VALUE> {
    #[inline]
    fn unpack<R>(mut reader: R) -> Result<Self, R::Error>
    where
        R: BitReader,
    {
        if VALUE != reader.unpack::<bool>()? {
            Err(Error::custom(format!(
                "expected {:#b}, got {:#b}",
                VALUE as u8, !VALUE as u8
            )))
        } else {
            Ok(Self)
        }
    }
}

macro_rules! impl_bit_serde_for_integers {
    ($($t:tt)+) => {$(
        impl BitPack for $t {
            #[inline]
            fn pack<W>(&self, mut writer: W) -> Result<(), W::Error>
            where
                W: BitWriter,
            {
                writer.pack_as::<_, AsBytes>(self.to_be_bytes())?;
                Ok(())
            }
        }

        impl BitUnpack for $t {
            #[inline]
            fn unpack<R>(mut reader: R) -> Result<Self, R::Error>
            where
                R: BitReader,
            {
                reader.read_bytes_array().map(Self::from_be_bytes)
            }
        }

        impl<const BITS: usize> BitPackAs<$t> for NBits<BITS> {
            #[inline]
            fn pack_as<W>(source: &$t, mut writer: W) -> Result<(), W::Error>
            where
                W: BitWriter,
            {
                const BITS_SIZE: usize = bits_of::<$t>();
                assert!(BITS <= BITS_SIZE, "excessive bits for type");
                if BITS < BITS_SIZE - source.leading_zeros() as usize {
                    return Err(Error::custom(
                        format!("{source:#b} cannot be packed into {BITS} bits"),
                    ));
                }
                let bytes = source.to_be_bytes();
                let mut bits = bytes.as_bits::<Msb0>();
                bits = &bits[bits.len() - BITS..];
                writer.write_bitslice(bits)?;
                Ok(())
            }
        }

        impl<const BITS: usize> BitUnpackAs<$t> for NBits<BITS> {
            #[inline]
            fn unpack_as<R>(mut reader: R) -> Result<$t, R::Error>
            where
                R: BitReader,
            {
                const BITS_SIZE: usize = bits_of::<$t>();
                assert!(BITS <= BITS_SIZE, "excessive bits for type");
                let mut arr = [0u8; mem::size_of::<$t>()];
                reader.read_bits_into(&mut arr.as_mut_bits()[BITS_SIZE - BITS..])?;
                Ok($t::from_be_bytes(arr))
            }
        }
    )+};
}
impl_bit_serde_for_integers! {
    u8 u16 u32 u64 u128 usize
    i8 i16 i32 i64 i128 isize
}

macro_rules! const_uint {
    ($($vis:vis $name:ident<$typ:tt, $bits:literal>)+) => {$(
        $vis struct $name<const VALUE: $typ, const BITS: usize = $bits>;

        impl<const VALUE: $typ, const BITS: usize> BitPack for $name<VALUE, BITS> {
            #[inline]
            fn pack<W>(&self, mut writer: W) -> Result<(), W::Error>
            where
                W: BitWriter,
            {
                writer.pack_as::<_, NBits<BITS>>(VALUE)?;
                Ok(())
            }
        }

        impl<const VALUE: $typ, const BITS: usize> BitUnpack for $name<VALUE, BITS> {
            #[inline]
            fn unpack<R>(mut reader: R) -> Result<Self, R::Error>
            where
                R: BitReader,
            {
                let v = reader.unpack_as::<$typ, NBits<BITS>>()?;
                if v != VALUE {
                    return Err(Error::custom(format!(
                        "expected {VALUE:#b}, got: {v:#b}"
                    )));
                }
                Ok(Self)
            }
        }
    )+};
}

const_uint! {
    pub ConstU8  <u8,   8>
    pub ConstU16 <u16,  16>
    pub ConstU32 <u32,  32>
    pub ConstU64 <u64,  64>
    pub ConstU128<u128, 128>
}

#[cfg(test)]
mod tests {
    use bitvec::{bits, order::Msb0};
    use num_bigint::BigUint;

    use crate::{
        pack, pack_as,
        tests::{assert_pack_unpack_as_eq, assert_pack_unpack_eq},
    };

    use super::*;

    #[test]
    fn store_uint() {
        assert_eq!(
            pack(0xFD_FE_u16).unwrap(),
            bits![u8, Msb0; 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0],
        )
    }

    #[test]
    fn serde_uint() {
        assert_pack_unpack_eq(12345_u32);
    }

    #[test]
    fn store_nbits_uint() {
        assert_eq!(
            pack_as::<_, NBits<7>>(0x7E).unwrap(),
            bits![u8, Msb0; 1, 1, 1, 1, 1, 1, 0],
        )
    }

    #[test]
    fn nbits_one_bit() {
        assert_eq!(pack_as::<_, NBits<1>>(0b1).unwrap(), pack(true).unwrap())
    }

    #[test]
    fn store_nbits_same_uint() {
        const N: u8 = 231;
        assert_eq!(pack(N).unwrap(), pack_as::<_, NBits<8>>(N).unwrap())
    }

    #[test]
    fn serde_nbits_uint() {
        assert_pack_unpack_as_eq::<u8, NBits<7>>(0x7E);
    }

    #[test]
    fn serde_big_nbits() {
        assert_pack_unpack_as_eq::<BigUint, NBits<100>>(12345_u64.into());
    }
}
