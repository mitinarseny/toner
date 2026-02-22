use std::borrow::Cow;

use bitvec::{order::Msb0, slice::BitSlice, vec::BitVec, view::AsBits};

use crate::{
    r#as::BorrowCow,
    de::{BitReader, BitReaderExt, r#as::BitUnpackAs},
    ser::{BitPack, BitWriter, BitWriterExt, r#as::BitPackAs},
};

/// **Ser**ialize value by taking a reference to [`BitSlice`] on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AsBitSlice;

impl<T> BitPackAs<T> for AsBitSlice
where
    T: AsRef<BitSlice<u8, Msb0>>,
{
    #[inline]
    fn pack_as<W>(source: &T, writer: &mut W) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        source.as_ref().pack(writer)
    }
}

/// **Ser**ialize value by taking a reference to `[u8]` on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AsBytes;

impl<T> BitPackAs<T> for AsBytes
where
    T: AsRef<[u8]>,
{
    #[inline]
    fn pack_as<W>(source: &T, writer: &mut W) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        source.as_bits().pack(writer)
    }
}

/// **De**/**ser**ialize value from/into exactly `N` bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NBits<const BITS: usize>;

/// **De**/**ser**ialize bits by prefixing its length with `N`-bit integer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VarBits<const BITS_FOR_LEN: usize>;

impl<const BITS_FOR_LEN: usize, T> BitPackAs<T> for VarBits<BITS_FOR_LEN>
where
    T: AsRef<BitSlice<u8, Msb0>>,
{
    #[inline]
    fn pack_as<W>(source: &T, writer: &mut W) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        let source = source.as_ref();
        writer
            .pack_as::<_, NBits<BITS_FOR_LEN>>(source.len())?
            .pack(source)?;
        Ok(())
    }
}

impl<'de: 'a, 'a, const BITS_FOR_LEN: usize> BitUnpackAs<'de, Cow<'a, BitSlice<u8, Msb0>>>
    for VarBits<BITS_FOR_LEN>
{
    #[inline]
    fn unpack_as<R>(reader: &mut R) -> Result<Cow<'a, BitSlice<u8, Msb0>>, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        let num_bits = reader.unpack_as::<_, NBits<BITS_FOR_LEN>>()?;
        reader.unpack_as_with::<_, BorrowCow>(num_bits)
    }
}

impl<'de, const BITS_FOR_LEN: usize> BitUnpackAs<'de, BitVec<u8, Msb0>> for VarBits<BITS_FOR_LEN> {
    #[inline]
    fn unpack_as<R>(reader: &mut R) -> Result<BitVec<u8, Msb0>, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        reader
            .unpack_as::<Cow<BitSlice<u8, Msb0>>, Self>()
            .map(Cow::into_owned)
    }
}

/// **De**/**ser**ialize bytes by prefixing its length with `N`-bit integer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VarBytes<const BITS_FOR_BYTES_LEN: usize>;

impl<const BITS_FOR_BYTES_LEN: usize, T> BitPackAs<T> for VarBytes<BITS_FOR_BYTES_LEN>
where
    T: AsRef<[u8]> + ?Sized,
{
    #[inline]
    fn pack_as<W>(source: &T, writer: &mut W) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        let source = source.as_ref();
        writer
            .pack_as::<_, NBits<BITS_FOR_BYTES_LEN>>(source.len())?
            .pack_as::<_, AsBytes>(source)?;
        Ok(())
    }
}

impl<'de: 'a, 'a, const BITS_FOR_BYTES_LEN: usize> BitUnpackAs<'de, Cow<'a, [u8]>>
    for VarBytes<BITS_FOR_BYTES_LEN>
{
    #[inline]
    fn unpack_as<R>(reader: &mut R) -> Result<Cow<'a, [u8]>, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        let num_bytes = reader.unpack_as::<_, NBits<BITS_FOR_BYTES_LEN>>()?;
        reader.unpack_as_with::<_, BorrowCow>(num_bytes)
    }
}

impl<'de, const BITS_FOR_BYTES_LEN: usize> BitUnpackAs<'de, Vec<u8>>
    for VarBytes<BITS_FOR_BYTES_LEN>
{
    #[inline]
    fn unpack_as<R>(reader: &mut R) -> Result<Vec<u8>, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        reader.unpack_as::<Cow<[u8]>, Self>().map(Cow::into_owned)
    }
}
