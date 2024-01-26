use crate::{BitPack, BitPackAs, BitReader, BitUnpack, BitUnpackAs, BitWriter};

pub struct Same;

impl<T> BitPackAs<T> for Same
where
    T: BitPack,
{
    #[inline]
    fn pack_as<W>(source: &T, writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        source.pack(writer)
    }
}

impl<T> BitUnpackAs<T> for Same
where
    T: BitUnpack,
{
    #[inline]
    fn unpack_as<R>(reader: R) -> Result<T, R::Error>
    where
        R: BitReader,
    {
        T::unpack(reader)
    }
}
