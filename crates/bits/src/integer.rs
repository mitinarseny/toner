//! Collection of **de**/**ser**ialization helpers for integers
use core::mem;

use bitvec::{
    mem::bits_of,
    order::Msb0,
    view::{AsBits, AsMutBits},
};

use crate::{
    Error,
    r#as::{AsBytes, NBits},
    de::{BitReader, BitReaderExt, BitUnpack, r#as::BitUnpackAs},
    ser::{BitPack, BitWriter, BitWriterExt, r#as::BitPackAs},
};

/// Constant version of `bool`
///
/// ## Deserialization
///
/// Reads `bool` and returns an error if it didn't match the
/// type parameter.
///
/// ```rust
/// # use tlbits::{
/// #   bitvec::{bits, order::Msb0},
/// #   de::{BitReaderExt},
/// #   Error,
/// #   integer::ConstBit,
/// #   StringError,
/// # };
/// # fn main() -> Result<(), StringError> {
/// # let mut reader = bits![u8, Msb0; 1, 1];
/// reader.unpack::<ConstBit<true>>()?;
/// // is equivalent of:
/// if !reader.unpack::<bool>()? {
///     return Err(Error::custom("expected 1, got 0"));
/// }
/// # Ok(())
/// # }
/// ```
///
/// ## Serialization
///
/// Writes `bool` specified in type parameter.
///
/// ```rust
/// # use tlbits::{
/// #   bitvec::{bits, vec::BitVec, order::Msb0},
/// #   integer::ConstBit,
/// #   ser::BitWriterExt,
/// #   StringError,
/// # };
/// # fn main() -> Result<(), StringError> {
/// # let mut writer = BitVec::<u8, Msb0>::new();
/// writer.pack(ConstBit::<true>)?;
/// // is equivalent of:
/// writer.pack(true)?;
/// # assert_eq!(writer, bits![u8, Msb0; 1, 1]);
/// # Ok(())
/// # }
/// ```
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
                let bits = bytes.as_bits::<Msb0>();
                writer.write_bitslice(&bits[bits.len() - BITS..])?;
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
                let arr_bits = &mut arr.as_mut_bits()[BITS_SIZE - BITS..];
                if reader.read_bits_into(arr_bits)? != arr_bits.len() {
                    return Err(Error::custom("EOF"));
                }
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
        #[doc = concat!("Constant version of `", stringify!($typ), "`")]
        /// ## Deserialization
        #[doc = concat!(
            "Reads `", stringify!($typ), "` and returns an error
            if it didn't match the type parameter.",
        )]
        ///
        /// ```rust
        /// # use tlbits::{
        /// #   bitvec::{vec::BitVec, order::Msb0},
        /// #   de::BitReaderExt,
        /// #   Error,
        #[doc = concat!("# integer::", stringify!($name), ",")]
        /// #   ser::BitWriterExt,
        /// #   StringError,
        /// # };
        /// # fn main() -> Result<(), StringError> {
        /// # let mut buff = BitVec::<u8, Msb0>::new();
        #[doc = concat!("# buff.pack::<[", stringify!($typ), "; 2]>([123; 2])?;")]
        /// # let mut reader = buff.as_bitslice();
        #[doc = concat!("reader.unpack::<", stringify!($name), "<123>>()?;")]
        /// // is equivalent of:
        #[doc = concat!("let got: ", stringify!($typ), " = reader.unpack()?;")]
        /// if got != 123 {
        ///     return Err(Error::custom(format!("expected 123, got {got}")));
        /// }
        /// # Ok(())
        /// # }
        /// ```
        ///
        /// ## Serialization
        ///
        #[doc = concat!(
            "Writes `", stringify!($typ), "` as specified in type parameter."
        )]
        ///
        /// ```rust
        /// # use tlbits::{
        /// #   bitvec::{bits, vec::BitVec, order::Msb0},
        /// #   de::BitReaderExt,
        #[doc = concat!("# integer::", stringify!($name), ",")]
        /// #   ser::BitWriterExt,
        /// #   StringError,
        /// # };
        /// # fn main() -> Result<(), StringError> {
        /// # let mut writer = BitVec::<u8, Msb0>::new();
        #[doc = concat!("writer.pack(", stringify!($name), "::<123>)?;")]
        /// // is equivalent of:
        #[doc = concat!("writer.pack::<", stringify!($typ), ">(123)?;")]
        /// # let mut reader = writer.as_bitslice();
        #[doc = concat!(
            "# assert_eq!(reader.unpack::<[", stringify!($typ), "; 2]>()?, [123; 2]);"
        )]
        /// # Ok(())
        /// # }
        /// ```
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
    pub ConstI8  <i8,   8>
    pub ConstU16 <u16,  16>
    pub ConstI16 <i16,  16>
    pub ConstU32 <u32,  32>
    pub ConstI32 <i32,  32>
    pub ConstU64 <u64,  64>
    pub ConstI64 <i64,  64>
    pub ConstU128<u128, 128>
    pub ConstI128<i128, 128>
}

#[cfg(test)]
mod tests {
    use bitvec::{bits, order::Msb0};
    use num_bigint::BigUint;

    use crate::{
        ser::{r#as::pack_as, pack},
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
