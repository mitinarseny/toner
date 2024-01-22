use either::Either;

use crate::{
    CellBuilder, CellParser, Result, TLBDeserialize, TLBDeserializeAs, TLBDeserializeAsWrap,
    TLBSerialize, TLBSerializeAs, TLBSerializeAsWrap,
};

impl<L, R> TLBSerialize for Either<L, R>
where
    L: TLBSerialize,
    R: TLBSerialize,
{
    #[inline]
    fn store(&self, builder: &mut CellBuilder) -> Result<()> {
        match self {
            Self::Left(l) => builder.store(false)?.store(l),
            Self::Right(r) => builder.store(true)?.store(r),
        }?;
        Ok(())
    }
}

impl<T> TLBSerializeAs<Option<T>> for Either<(), T>
where
    T: TLBSerialize,
{
    #[inline]
    fn store_as(source: &Option<T>, builder: &mut CellBuilder) -> Result<()> {
        match source.as_ref() {
            None => Either::Left(()),
            Some(v) => Either::Right(v),
        }
        .store(builder)
    }
}

/// [Maybe](https://docs.ton.org/develop/data-formats/tl-b-types#maybe)
impl<T> TLBSerialize for Option<T>
where
    T: TLBSerialize,
{
    #[inline]
    fn store(&self, builder: &mut CellBuilder) -> Result<()> {
        TLBSerializeAsWrap::<_, Either<(), T>>::new(self).store(builder)
    }
}

impl<'de, L, R> TLBDeserialize<'de> for Either<L, R>
where
    L: TLBDeserialize<'de>,
    R: TLBDeserialize<'de>,
{
    #[inline]
    fn parse(parser: &mut CellParser<'de>) -> Result<Self> {
        match parser.parse()? {
            false => parser.parse().map(Either::Left),
            true => parser.parse().map(Either::Right),
        }
    }
}

impl<'de, T> TLBDeserializeAs<'de, Option<T>> for Either<(), T>
where
    T: TLBDeserialize<'de>,
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de>) -> Result<Option<T>> {
        Either::<(), _>::parse(parser).map(Either::right)
    }
}

/// [Maybe](https://docs.ton.org/develop/data-formats/tl-b-types#maybe)
impl<'de, T> TLBDeserialize<'de> for Option<T>
where
    T: TLBDeserialize<'de>,
{
    #[inline]
    fn parse(parser: &mut CellParser<'de>) -> Result<Self> {
        TLBDeserializeAsWrap::<_, Either<(), T>>::parse(parser)
            .map(TLBDeserializeAsWrap::into_inner)
    }
}
