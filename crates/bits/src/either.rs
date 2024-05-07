use either::Either;

use crate::{
    BitPack, BitPackAs, BitPackAsWrap, BitReader, BitReaderExt, BitUnpack, BitUnpackAs,
    BitUnpackAsWrap, BitWriter, BitWriterExt, ResultExt, Same,
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
                BitPackAsWrap::<Left, AsLeft>::new,
                BitPackAsWrap::<Right, AsRight>::new,
            )
            .pack(writer)
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
            Either::<BitUnpackAsWrap<Left, AsLeft>, BitUnpackAsWrap<Right, AsRight>>::unpack(
                reader,
            )?
            .map_either(BitUnpackAsWrap::into_inner, BitUnpackAsWrap::into_inner),
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
            Some(v) => Either::Right(BitPackAsWrap::<T, As>::new(v)),
        }
        .pack(writer)
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
        Ok(Either::<(), BitUnpackAsWrap<T, As>>::unpack(reader)?
            .map_right(BitUnpackAsWrap::into_inner)
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
