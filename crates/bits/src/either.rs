use either::Either;

use crate::{
    de::{
        args::{r#as::BitUnpackAsWithArgs, BitUnpackWithArgs},
        r#as::{BitUnpackAs, UnpackAsWrap},
        BitReader, BitReaderExt, BitUnpack,
    },
    r#as::{args::NoArgs, Same},
    ser::{
        args::{r#as::BitPackAsWithArgs, BitPackWithArgs},
        r#as::{BitPackAs, PackAsWrap},
        BitPack, BitWriter, BitWriterExt,
    },
    ResultExt,
};

impl<L, R> BitPack for Either<L, R>
where
    L: BitPack,
    R: BitPack,
{
    #[inline]
    fn pack<W>(&self, mut writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        match self {
            Self::Left(l) => writer.pack(false).context("tag")?.pack(l).context("left")?,
            Self::Right(r) => writer.pack(true).context("tag")?.pack(r).context("right")?,
        };
        Ok(())
    }
}

impl<L, R> BitPackWithArgs for Either<L, R>
where
    L: BitPackWithArgs,
    R: BitPackWithArgs<Args = L::Args>,
{
    type Args = L::Args;

    #[inline]
    fn pack_with<W>(&self, mut writer: W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        match self {
            Self::Left(l) => writer
                .pack(false)
                .context("tag")?
                .pack_with(l, args)
                .context("left")?,
            Self::Right(r) => writer
                .pack(true)
                .context("tag")?
                .pack_with(r, args)
                .context("right")?,
        };
        Ok(())
    }
}

impl<Left, Right> BitUnpack for Either<Left, Right>
where
    Left: BitUnpack,
    Right: BitUnpack,
{
    #[inline]
    fn unpack<R>(mut reader: R) -> Result<Self, R::Error>
    where
        R: BitReader,
    {
        match reader.unpack().context("tag")? {
            false => reader.unpack().map(Either::Left).context("left"),
            true => reader.unpack().map(Either::Right).context("right"),
        }
    }
}

impl<Left, Right> BitUnpackWithArgs for Either<Left, Right>
where
    Left: BitUnpackWithArgs,
    Right: BitUnpackWithArgs<Args = Left::Args>,
{
    type Args = Left::Args;

    #[inline]
    fn unpack_with<R>(mut reader: R, args: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader,
    {
        match reader.unpack().context("tag")? {
            false => reader.unpack_with(args).map(Either::Left).context("left"),
            true => reader.unpack_with(args).map(Either::Right).context("right"),
        }
    }
}

impl<Left, Right, AsLeft, AsRight> BitPackAs<Either<Left, Right>> for Either<AsLeft, AsRight>
where
    AsLeft: BitPackAs<Left>,
    AsRight: BitPackAs<Right>,
{
    #[inline]
    fn pack_as<W>(source: &Either<Left, Right>, writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        source
            .as_ref()
            .map_either(
                PackAsWrap::<Left, AsLeft>::new,
                PackAsWrap::<Right, AsRight>::new,
            )
            .pack(writer)
    }
}

impl<Left, Right, AsLeft, AsRight> BitPackAsWithArgs<Either<Left, Right>>
    for Either<AsLeft, AsRight>
where
    AsLeft: BitPackAsWithArgs<Left>,
    AsRight: BitPackAsWithArgs<Right, Args = AsLeft::Args>,
{
    type Args = AsLeft::Args;

    #[inline]
    fn pack_as_with<W>(
        source: &Either<Left, Right>,
        writer: W,
        args: Self::Args,
    ) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        source
            .as_ref()
            .map_either(
                PackAsWrap::<Left, AsLeft>::new,
                PackAsWrap::<Right, AsRight>::new,
            )
            .pack_with(writer, args)
    }
}

impl<Left, Right, AsLeft, AsRight> BitUnpackAs<Either<Left, Right>> for Either<AsLeft, AsRight>
where
    AsLeft: BitUnpackAs<Left>,
    AsRight: BitUnpackAs<Right>,
{
    #[inline]
    fn unpack_as<R>(reader: R) -> Result<Either<Left, Right>, R::Error>
    where
        R: BitReader,
    {
        Ok(
            Either::<UnpackAsWrap<Left, AsLeft>, UnpackAsWrap<Right, AsRight>>::unpack(reader)?
                .map_either(UnpackAsWrap::into_inner, UnpackAsWrap::into_inner),
        )
    }
}

impl<Left, Right, AsLeft, AsRight> BitUnpackAsWithArgs<Either<Left, Right>>
    for Either<AsLeft, AsRight>
where
    AsLeft: BitUnpackAsWithArgs<Left>,
    AsRight: BitUnpackAsWithArgs<Right, Args = AsLeft::Args>,
{
    type Args = AsLeft::Args;

    #[inline]
    fn unpack_as_with<R>(reader: R, args: Self::Args) -> Result<Either<Left, Right>, R::Error>
    where
        R: BitReader,
    {
        Ok(
            Either::<UnpackAsWrap<Left, AsLeft>, UnpackAsWrap<Right, AsRight>>::unpack_with(
                reader, args,
            )?
            .map_either(UnpackAsWrap::into_inner, UnpackAsWrap::into_inner),
        )
    }
}

impl<T, As> BitPackAs<Option<T>> for Either<(), As>
where
    As: BitPackAs<T>,
{
    #[inline]
    fn pack_as<W>(source: &Option<T>, writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        match source.as_ref() {
            None => Either::Left(()),
            Some(v) => Either::Right(PackAsWrap::<T, As>::new(v)),
        }
        .pack(writer)
    }
}

impl<T, As> BitPackAsWithArgs<Option<T>> for Either<(), As>
where
    As: BitPackAsWithArgs<T>,
{
    type Args = As::Args;

    #[inline]
    fn pack_as_with<W>(source: &Option<T>, writer: W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        BitPackWithArgs::pack_with(
            &match source.as_ref() {
                None => Either::Left(PackAsWrap::<_, NoArgs<_>>::new(&())),
                Some(v) => Either::Right(PackAsWrap::<T, As>::new(v)),
            },
            writer,
            args,
        )
    }
}

impl<T, As> BitUnpackAs<Option<T>> for Either<(), As>
where
    As: BitUnpackAs<T>,
{
    #[inline]
    fn unpack_as<R>(reader: R) -> Result<Option<T>, R::Error>
    where
        R: BitReader,
    {
        Ok(Either::<(), UnpackAsWrap<T, As>>::unpack(reader)?
            .map_right(UnpackAsWrap::into_inner)
            .right())
    }
}

impl<T, As> BitUnpackAsWithArgs<Option<T>> for Either<(), As>
where
    As: BitUnpackAsWithArgs<T>,
{
    type Args = As::Args;

    #[inline]
    fn unpack_as_with<R>(mut reader: R, args: Self::Args) -> Result<Option<T>, R::Error>
    where
        R: BitReader,
    {
        Ok(reader
            .unpack_as_with::<Either<(), T>, Either<NoArgs<_>, As>>(args)?
            .right())
    }
}

/// [Maybe](https://docs.ton.org/develop/data-formats/tl-b-types#maybe)
impl<T> BitPack for Option<T>
where
    T: BitPack,
{
    #[inline]
    fn pack<W>(&self, mut writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        writer.pack_as::<_, Either<(), Same>>(self.as_ref())?;
        Ok(())
    }
}

/// [Maybe](https://docs.ton.org/develop/data-formats/tl-b-types#maybe)
impl<T> BitPackWithArgs for Option<T>
where
    T: BitPackWithArgs,
{
    type Args = T::Args;

    #[inline]
    fn pack_with<W>(&self, mut writer: W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        writer.pack_as_with::<_, Either<(), Same>>(self.as_ref(), args)?;
        Ok(())
    }
}

/// [Maybe](https://docs.ton.org/develop/data-formats/tl-b-types#maybe)
impl<T> BitUnpack for Option<T>
where
    T: BitUnpack,
{
    #[inline]
    fn unpack<R>(mut reader: R) -> Result<Self, R::Error>
    where
        R: BitReader,
    {
        reader.unpack_as::<_, Either<(), Same>>()
    }
}

/// [Maybe](https://docs.ton.org/develop/data-formats/tl-b-types#maybe)
impl<T> BitUnpackWithArgs for Option<T>
where
    T: BitUnpackWithArgs,
{
    type Args = T::Args;

    #[inline]
    fn unpack_with<R>(mut reader: R, args: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader,
    {
        reader.unpack_as_with::<_, Either<(), Same>>(args)
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::assert_pack_unpack_eq;

    use super::*;

    #[test]
    fn either_left() {
        assert_pack_unpack_eq(Either::<u8, u16>::Left(1));
    }

    #[test]
    fn either_right() {
        assert_pack_unpack_eq(Either::<u8, u16>::Right(2));
    }

    #[test]
    fn none() {
        assert_pack_unpack_eq::<Option<u8>>(None);
    }

    #[test]
    fn some() {
        assert_pack_unpack_eq(Some(123));
    }
}
