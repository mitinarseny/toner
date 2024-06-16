use bitvec::{order::Msb0, slice::BitSlice, view::AsBits};

use crate::{
    de::{r#as::BitUnpackAs, BitReader, BitReaderExt},
    ser::{r#as::BitPackAs, BitPack, BitWriter, BitWriterExt},
};

/// **Ser**ialize value by taking a reference to [`BitSlice`] on it.
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

/// **Ser**ialize value by taking a reference to `[u8]` on it.
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

/// **De**/**ser**ialize value from/into exactly `N` bits.
pub struct NBits<const BITS: usize>;

/// **De**/**ser**ialize bytes by prefixing its length with `N`-bit integer.
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
            .pack_as::<_, NBits<BITS_FOR_BYTES_LEN>>(source.len())?
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
        let num_bytes = reader.unpack_as::<_, NBits<BITS_FOR_BYTES_LEN>>()?;
        reader.read_bytes_vec(num_bytes)
    }
}
