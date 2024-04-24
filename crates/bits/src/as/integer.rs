use bitvec::{order::Msb0, vec::BitVec, view::AsBits};
use num_bigint::{BigInt, BigUint};

use crate::{
    BitPackAs, BitReader, BitReaderExt, BitUnpackAs, BitWriter, BitWriterExt, Error, NBits,
    VarBytes,
};

impl<const BITS: usize> BitPackAs<BigUint> for NBits<BITS> {
    #[inline]
    fn pack_as<W>(source: &BigUint, mut writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        let used_bits = source.bits() as usize;
        if BITS < used_bits {
            return Err(Error::custom(
                "{source:#b} cannot be packed into {BITS} bits",
            ));
        }

        writer.repeat_bit(BITS - used_bits, false)?;

        let bytes = source.to_bytes_be();
        let mut bits = bytes.as_bits::<Msb0>();
        bits = &bits[bits.len() - used_bits..];
        writer.with_bits(bits)?;
        Ok(())
    }
}

impl<const BITS: usize> BitUnpackAs<BigUint> for NBits<BITS> {
    #[inline]
    fn unpack_as<R>(mut reader: R) -> Result<BigUint, R::Error>
    where
        R: BitReader,
    {
        let mut bits = reader.read_bitvec(BITS)?;
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
            return Err(Error::custom(
                "{source:#b} cannot be packed into {BITS} bits",
            ));
        }

        writer.repeat_bit(BITS - used_bits, false)?;

        let bytes = source.to_signed_bytes_be();
        let mut bits = bytes.as_bits::<Msb0>();
        bits = &bits[bits.len() - used_bits..];
        writer.with_bits(bits)?;
        Ok(())
    }
}

impl<const BITS: usize> BitUnpackAs<BigInt> for NBits<BITS> {
    #[inline]
    fn unpack_as<R>(mut reader: R) -> Result<BigInt, R::Error>
    where
        R: BitReader,
    {
        let mut bits = reader.read_bitvec(BITS)?;
        let total_bits = (BITS + 7) & !7;
        bits.resize(total_bits, false);
        bits.shift_right(total_bits - BITS);
        Ok(BigInt::from_signed_bytes_be(bits.as_raw_slice()))
    }
}

pub struct VarUint<const BITS_FOR_BYTES_LEN: usize>;

impl<const BITS_FOR_BYTES_LEN: usize> BitPackAs<BigUint> for VarUint<BITS_FOR_BYTES_LEN> {
    #[inline]
    fn pack_as<W>(source: &BigUint, mut writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        writer.pack_as::<_, VarBytes<BITS_FOR_BYTES_LEN>>(source.to_bytes_be())?;
        Ok(())
    }
}

impl<const BITS_FOR_BYTES_LEN: usize> BitUnpackAs<BigUint> for VarUint<BITS_FOR_BYTES_LEN> {
    #[inline]
    fn unpack_as<R>(mut reader: R) -> Result<BigUint, R::Error>
    where
        R: BitReader,
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

pub struct VarInt<const BITS_FOR_BYTES_LEN: usize>;

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

impl<const BITS_FOR_BYTES_LEN: usize> BitUnpackAs<BigInt> for VarInt<BITS_FOR_BYTES_LEN> {
    #[inline]
    fn unpack_as<R>(mut reader: R) -> Result<BigInt, R::Error>
    where
        R: BitReader,
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
