use core::fmt::Display;

use crate::{
    CellBuilder, CellBuilderError, CellDeserialize, CellDeserializeAs, CellParser, CellParserError,
    CellSerialize, CellSerializeAs, Error, FromInto, FromIntoRef, TryFromInto,
};

impl<T, As> CellSerializeAs<T> for FromInto<As>
where
    T: Into<As> + Clone,
    As: CellSerialize,
{
    #[inline]
    fn store_as(source: &T, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        source.clone().into().store(builder)
    }
}

impl<'de, T, As> CellDeserializeAs<'de, T> for FromInto<As>
where
    As: Into<T> + CellDeserialize<'de>,
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de>) -> Result<T, CellParserError<'de>> {
        As::parse(parser).map(Into::into)
    }
}

impl<T, As> CellSerializeAs<T> for FromIntoRef<As>
where
    for<'a> &'a T: Into<As>,
    As: CellSerialize,
{
    #[inline]
    fn store_as(source: &T, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        source.into().store(builder)
    }
}

impl<'de, T, As> CellDeserializeAs<'de, T> for FromIntoRef<As>
where
    As: Into<T> + CellDeserialize<'de>,
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de>) -> Result<T, CellParserError<'de>> {
        As::parse(parser).map(Into::into)
    }
}

impl<T, As> CellSerializeAs<T> for TryFromInto<As>
where
    T: TryInto<As> + Clone,
    <T as TryInto<As>>::Error: Display,
    As: CellSerialize,
{
    #[inline]
    fn store_as(source: &T, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        source
            .clone()
            .try_into()
            .map_err(Error::custom)?
            .store(builder)
    }
}

impl<'de, T, As> CellDeserializeAs<'de, T> for TryFromInto<As>
where
    As: TryInto<T> + CellDeserialize<'de>,
    <As as TryInto<T>>::Error: Display,
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de>) -> Result<T, CellParserError<'de>> {
        As::parse(parser)?.try_into().map_err(Error::custom)
    }
}
