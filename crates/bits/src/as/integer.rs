use core::{
    fmt::{Binary, LowerHex},
    mem::size_of,
};

use bitvec::{mem::bits_of, order::Msb0, vec::BitVec, view::AsBits};
use num_bigint::{BigInt, BigUint};
use num_traits::{PrimInt, ToBytes};

use crate::{
    Error,
    de::{BitReader, BitReaderExt, args::r#as::BitUnpackAsWithArgs, r#as::BitUnpackAs},
    ser::{BitWriter, BitWriterExt, args::r#as::BitPackAsWithArgs, r#as::BitPackAs},
};

use super::{NBits, VarBytes};

impl<const BITS: usize> BitPackAs<BigUint> for NBits<BITS> {
    #[inline]
    fn pack_as<W>(source: &BigUint, mut writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        let used_bits = source.bits() as usize;
        if BITS < used_bits {
            return Err(Error::custom(format!(
                "{source:#b} cannot be packed into {BITS} bits"
            )));
        }

        writer.repeat_bit(BITS - used_bits, false)?;

        let bytes = source.to_bytes_be();
        let mut bits = bytes.as_bits::<Msb0>();
        bits = &bits[bits.len() - used_bits..];
        writer.pack(bits)?;
        Ok(())
    }
}

impl<'de, const BITS: usize> BitUnpackAs<'de, BigUint> for NBits<BITS> {
    #[inline]
    fn unpack_as<R>(mut reader: R) -> Result<BigUint, R::Error>
    where
        R: BitReader<'de>,
    {
        let mut bits: BitVec<u8, Msb0> = reader.unpack_with(BITS)?;
        let total_bits = (BITS + 7) & !7;
        bits.resize(total_bits, false);
        bits.shift_right(total_bits - BITS);
        Ok(BigUint::from_bytes_be(bits.as_raw_slice()))
    }
}

impl<const BITS: usize> BitPackAs<BigInt> for NBits<BITS> {
    #[inline]
    fn pack_as<W>(source: &BigInt, mut writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        let used_bits = source.bits() as usize;
        if BITS < used_bits {
            return Err(Error::custom(format!(
                "{source:#b} cannot be packed into {BITS} bits"
            )));
        }

        writer.repeat_bit(BITS - used_bits, false)?;

        let bytes = source.to_signed_bytes_be();
        let mut bits = bytes.as_bits::<Msb0>();
        bits = &bits[bits.len() - used_bits..];
        writer.pack(bits)?;
        Ok(())
    }
}

impl<'de, const BITS: usize> BitUnpackAs<'de, BigInt> for NBits<BITS> {
    #[inline]
    fn unpack_as<R>(mut reader: R) -> Result<BigInt, R::Error>
    where
        R: BitReader<'de>,
    {
        let mut bits: BitVec<u8, Msb0> = reader.unpack_with(BITS)?;
        let total_bits = (BITS + 7) & !7;
        bits.resize(total_bits, false);
        bits.shift_right(total_bits - BITS);
        Ok(BigInt::from_signed_bytes_be(bits.as_raw_slice()))
    }
}

/// Adapter for [`Var[U]Integer n`](https://docs.ton.org/develop/data-formats/msg-tlb#varuinteger-n)
/// where `n` is *constant*.
///
/// ```tlb
/// var_uint$_ {n:#} len:(#< n) value:(uint (len * 8)) = VarUInteger n;
/// var_int$_ {n:#} len:(#< n) value:(int (len * 8)) = VarInteger n;
/// ```
/// See [`VarNBits`] for *dynamic* version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VarInt<const BITS_FOR_BYTES_LEN: usize>;

impl<const BITS_FOR_BYTES_LEN: usize> BitPackAs<BigUint> for VarInt<BITS_FOR_BYTES_LEN> {
    #[inline]
    fn pack_as<W>(source: &BigUint, mut writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        let bytes = if source != &BigUint::ZERO {
            source.to_bytes_be()
        } else {
            // BigUint::to_bytes_be() returns [0] instead of []
            Vec::new()
        };
        writer.pack_as::<_, VarBytes<BITS_FOR_BYTES_LEN>>(bytes)?;
        Ok(())
    }
}

impl<'de, const BITS_FOR_BYTES_LEN: usize> BitUnpackAs<'de, BigUint>
    for VarInt<BITS_FOR_BYTES_LEN>
{
    #[inline]
    fn unpack_as<R>(mut reader: R) -> Result<BigUint, R::Error>
    where
        R: BitReader<'de>,
    {
        let mut bits = BitVec::<u8, Msb0>::from_vec(
            reader.unpack_as::<Vec<u8>, VarBytes<BITS_FOR_BYTES_LEN>>()?,
        );
        let total_bits = (bits.len() + 7) & !7;
        let shift = total_bits - bits.len();
        bits.resize(total_bits, false);
        bits.shift_right(shift);
        Ok(BigUint::from_bytes_be(bits.as_raw_slice()))
    }
}

impl<const BITS_FOR_BYTES_LEN: usize> BitPackAs<BigInt> for VarInt<BITS_FOR_BYTES_LEN> {
    #[inline]
    fn pack_as<W>(source: &BigInt, mut writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        writer.pack_as::<_, VarBytes<BITS_FOR_BYTES_LEN>>(source.to_signed_bytes_be())?;
        Ok(())
    }
}

impl<'de, const BITS_FOR_BYTES_LEN: usize> BitUnpackAs<'de, BigInt> for VarInt<BITS_FOR_BYTES_LEN> {
    #[inline]
    fn unpack_as<R>(mut reader: R) -> Result<BigInt, R::Error>
    where
        R: BitReader<'de>,
    {
        let mut bits = BitVec::<u8, Msb0>::from_vec(
            reader.unpack_as::<Vec<u8>, VarBytes<BITS_FOR_BYTES_LEN>>()?,
        );
        let total_bits = (bits.len() + 7) & !7;
        let shift = total_bits - bits.len();
        bits.resize(total_bits, false);
        bits.shift_right(shift);
        Ok(BigInt::from_signed_bytes_be(bits.as_raw_slice()))
    }
}

/// Adapter for [`Var[U]Integer (n * 8)`](https://docs.ton.org/develop/data-formats/msg-tlb#varuinteger-n) where `n` is *dynamic*.
/// ```tlb
/// var_uint$_ {n:#} len:(#< n) value:(uint (len * 8)) = VarUInteger n;
/// var_int$_ {n:#} len:(#< n) value:(int (len * 8)) = VarInteger n;
/// ```
/// See [`VarInt`] for *constant* version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VarNBits;

impl<T> BitPackAsWithArgs<T> for VarNBits
where
    T: PrimInt + Binary + ToBytes,
{
    /// number of bits
    type Args = u32;

    #[inline]
    fn pack_as_with<W>(source: &T, mut writer: W, num_bits: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        let size_bits: u32 = bits_of::<T>() as u32;
        let leading_zeroes = source.leading_zeros();
        let used_bits = size_bits - leading_zeroes;
        if num_bits < used_bits {
            return Err(Error::custom(format!(
                "{source:0b} cannot be packed into {num_bits} bits",
            )));
        }
        let arr = source.to_be_bytes();
        let bits = arr.as_bits();
        writer.write_bitslice(&bits[bits.len() - num_bits as usize..])?;
        Ok(())
    }
}

impl<'de, T> BitUnpackAsWithArgs<'de, T> for VarNBits
where
    T: PrimInt,
{
    /// number of bits
    type Args = u32;

    #[inline]
    fn unpack_as_with<R>(mut reader: R, num_bits: Self::Args) -> Result<T, R::Error>
    where
        R: BitReader<'de>,
    {
        let size_bits: u32 = bits_of::<T>() as u32;
        if num_bits > size_bits {
            return Err(Error::custom("excessive bits for the type"));
        }
        let mut v: T = T::zero();
        for bit in reader.unpack_iter::<bool>().take(num_bits as usize) {
            v = v << 1;
            v = v | if bit? { T::one() } else { T::zero() };
        }
        Ok(v)
    }
}

/// Adapter for [`Var[U]Integer n`](https://docs.ton.org/develop/data-formats/msg-tlb#varuinteger-n) where `n` is *dynamic*.
/// ```tlb
/// var_uint$_ {n:#} len:(#< n) value:(uint (len * 8)) = VarUInteger n;
/// var_int$_ {n:#} len:(#< n) value:(int (len * 8)) = VarInteger n;
/// ```
/// See [`VarInt`] for *constant* version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VarNBytes;

impl<T> BitPackAsWithArgs<T> for VarNBytes
where
    T: PrimInt + LowerHex + ToBytes,
{
    /// number of bytes
    type Args = u32;

    #[inline]
    fn pack_as_with<W>(source: &T, mut writer: W, num_bytes: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        let size_bytes: u32 = size_of::<T>() as u32;
        let leading_zeroes = source.leading_zeros();
        let used_bytes = size_bytes - leading_zeroes / 8;
        if num_bytes < used_bytes {
            return Err(Error::custom(format!(
                "{source:0x} cannot be packed into {num_bytes} bytes",
            )));
        }
        let arr = source.to_be_bytes();
        let bytes = arr.as_ref();
        writer.write_bitslice((&bytes[bytes.len() - num_bytes as usize..]).as_bits())?;
        Ok(())
    }
}

impl<'de, T> BitUnpackAsWithArgs<'de, T> for VarNBytes
where
    T: PrimInt,
{
    /// number of bytes
    type Args = u32;

    #[inline]
    fn unpack_as_with<R>(mut reader: R, num_bytes: Self::Args) -> Result<T, R::Error>
    where
        R: BitReader<'de>,
    {
        let size_bytes: u32 = size_of::<T>() as u32;
        if num_bytes > size_bytes {
            return Err(Error::custom("excessive bits for type"));
        }
        let mut v: T = T::zero();
        for byte in reader.unpack_iter::<u8>().take(num_bytes as usize) {
            v = v << 8;
            v = v | T::from(byte?).unwrap();
        }
        Ok(v)
    }
}
