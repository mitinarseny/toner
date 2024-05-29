use crate::{
    BitPack, BitPackAs, BitPackAsWithArgs, BitPackWithArgs, BitReader, BitUnpack, BitUnpackAs,
    BitUnpackAsWithArgs, BitUnpackWithArgs, BitWriter,
};

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

impl<T> BitUnpackAsWithArgs<T> for Same
where
    T: BitUnpackWithArgs,
{
    type Args = T::Args;

    #[inline]
    fn unpack_as_with<R>(reader: R, args: Self::Args) -> Result<T, R::Error>
    where
        R: BitReader,
    {
        T::unpack_with(reader, args)
    }
}
