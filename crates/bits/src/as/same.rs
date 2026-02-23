use crate::{
    de::{BitReader, BitUnpack, BitUnpackAs},
    ser::{BitPack, BitPackAs, BitWriter},
};

/// Adapter to convert from `*As` to regular **de**/**ser**ialization traits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Same;

impl<T> BitPackAs<T> for Same
where
    T: BitPack + ?Sized,
{
    type Args = T::Args;

    #[inline]
    fn pack_as<W>(source: &T, writer: &mut W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        T::pack(source, writer, args)
    }
}

impl<'de, T> BitUnpackAs<'de, T> for Same
where
    T: BitUnpack<'de> + ?Sized,
{
    type Args = T::Args;

    #[inline]
    fn unpack_as<R>(reader: &mut R, args: Self::Args) -> Result<T, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        T::unpack(reader, args)
    }
}
