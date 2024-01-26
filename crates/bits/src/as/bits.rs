use bitvec::{order::Msb0, slice::BitSlice, view::AsBits};

use crate::{
    BitPack, BitPackAs, BitReader, BitReaderExt, BitUnpackAs, BitWriter, BitWriterExt, ResultExt,
};

pub struct AsBitSlice;

impl<T> BitPackAs<T> for AsBitSlice
where
    T: AsRef<BitSlice<u8, Msb0>>,
{
    #[inline]
    fn pack_as<W>(source: &T, writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        source.as_ref().pack(writer)
    }
}

pub struct AsBytes;

impl<T> BitPackAs<T> for AsBytes
where
    T: AsRef<[u8]>,
{
    #[inline]
    fn pack_as<W>(source: &T, writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        source.as_bits().pack(writer)
    }
}

pub struct NBits<const BITS: usize>;

pub struct VarBytes<const BITS_FOR_BYTES_LEN: usize>;

impl<const BITS_FOR_BYTES_LEN: usize, T> BitPackAs<T> for VarBytes<BITS_FOR_BYTES_LEN>
where
    T: AsRef<[u8]> + ?Sized,
{
    #[inline]
    fn pack_as<W>(source: &T, mut writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        let source = source.as_ref();
        writer
            .pack_as::<_, NBits<BITS_FOR_BYTES_LEN>>(source.len())
            .context("length")?
            .pack_as::<_, AsBytes>(source)?;
        Ok(())
    }
}

impl<const BITS_FOR_BYTES_LEN: usize> BitUnpackAs<Vec<u8>> for VarBytes<BITS_FOR_BYTES_LEN> {
    #[inline]
    fn unpack_as<R>(mut reader: R) -> Result<Vec<u8>, R::Error>
    where
        R: BitReader,
    {
        let num_bytes = reader
            .unpack_as::<_, NBits<BITS_FOR_BYTES_LEN>>()
            .context("length")?;
        reader.read_bytes_vec(num_bytes)
    }
}
