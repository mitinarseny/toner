use crate::{
    de::{
        BitReader, BitUnpack,
        args::{BitUnpackWithArgs, r#as::BitUnpackAsWithArgs},
        r#as::BitUnpackAs,
    },
    ser::{
        BitPack, BitWriter,
        args::{BitPackWithArgs, r#as::BitPackAsWithArgs},
        r#as::BitPackAs,
    },
};

/// Adapter to convert from `*As` to regular **de**/**ser**ialization traits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

impl<T> BitPackAsWithArgs<T> for Same
where
    T: BitPackWithArgs,
{
    type Args = T::Args;

    #[inline]
    fn pack_as_with<W>(source: &T, writer: W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        T::pack_with(source, writer, args)
    }
}

impl<'de, T> BitUnpackAs<'de, T> for Same
where
    T: BitUnpack<'de>,
{
    #[inline]
    fn unpack_as<R>(reader: R) -> Result<T, R::Error>
    where
        R: BitReader<'de>,
    {
        T::unpack(reader)
    }
}

impl<'de, T> BitUnpackAsWithArgs<'de, T> for Same
where
    T: BitUnpackWithArgs<'de>,
{
    type Args = T::Args;

    #[inline]
    fn unpack_as_with<R>(reader: R, args: Self::Args) -> Result<T, R::Error>
    where
        R: BitReader<'de>,
    {
        T::unpack_with(reader, args)
    }
}
