use either::Either;

use crate::{
    BitReaderExt, BitWriterExt, CellBuilder, CellBuilderError, CellDeserialize, CellDeserializeAs,
    CellDeserializeAsWrap, CellParser, CellParserError, CellSerialize, CellSerializeAs,
    CellSerializeAsWrap, ResultExt, StringError,
};

impl<L, R> CellSerialize for Either<L, R>
where
    L: CellSerialize,
    R: CellSerialize,
{
    #[inline]
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        match self {
            Self::Left(l) => builder
                .pack(false)
                .context("tag")?
                .store(l)
                .context("left")?,
            Self::Right(r) => builder
                .pack(true)
                .context("tag")?
                .store(r)
                .context("right")?,
        };
        Ok(())
    }
}

impl<'de, Left, Right> CellDeserialize<'de> for Either<Left, Right>
where
    Left: CellDeserialize<'de>,
    Right: CellDeserialize<'de>,
{
    #[inline]
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, CellParserError<'de>> {
        match parser.unpack().context("tag")? {
            false => parser.parse().map(Either::Left).context("left"),
            true => parser.parse().map(Either::Right).context("right"),
        }
    }
}

impl<Left, Right, AsLeft, AsRight> CellSerializeAs<Either<Left, Right>> for Either<AsLeft, AsRight>
where
    AsLeft: CellSerializeAs<Left>,
    AsRight: CellSerializeAs<Right>,
{
    #[inline]
    fn store_as(
        source: &Either<Left, Right>,
        builder: &mut CellBuilder,
    ) -> Result<(), CellBuilderError> {
        source
            .as_ref()
            .map_either(
                CellSerializeAsWrap::<Left, AsLeft>::new,
                CellSerializeAsWrap::<Right, AsRight>::new,
            )
            .store(builder)
    }
}

impl<'de, Left, Right, AsLeft, AsRight> CellDeserializeAs<'de, Either<Left, Right>>
    for Either<AsLeft, AsRight>
where
    AsLeft: CellDeserializeAs<'de, Left>,
    AsRight: CellDeserializeAs<'de, Right>,
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de>) -> Result<Either<Left, Right>, CellParserError<'de>> {
        Ok(
            Either::<CellDeserializeAsWrap<Left, AsLeft>, CellDeserializeAsWrap<Right, AsRight>>::parse(
                parser,
            )?
            .map_either(CellDeserializeAsWrap::into_inner, CellDeserializeAsWrap::into_inner),
        )
    }
}

impl<T> CellSerializeAs<Option<T>> for Either<(), T>
where
    T: CellSerialize,
{
    #[inline]
    fn store_as(source: &Option<T>, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        match source.as_ref() {
            None => Either::Left(()),
            Some(v) => Either::Right(v),
        }
        .store(builder)
    }
}

impl<'de, T> CellDeserializeAs<'de, Option<T>> for Either<(), T>
where
    T: CellDeserialize<'de>,
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de>) -> Result<Option<T>, StringError> {
        Self::parse(parser).map(Either::right)
    }
}

/// [Maybe](https://docs.ton.org/develop/data-formats/tl-b-types#maybe)
impl<T> CellSerialize for Option<T>
where
    T: CellSerialize,
{
    #[inline]
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        builder.store_as::<_, Either<(), &T>>(self.as_ref())?;
        Ok(())
    }
}

/// [Maybe](https://docs.ton.org/develop/data-formats/tl-b-types#maybe)
impl<'de, T> CellDeserialize<'de> for Option<T>
where
    T: CellDeserialize<'de>,
{
    #[inline]
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, StringError> {
        parser.parse_as::<_, Either<(), T>>()
    }
}
