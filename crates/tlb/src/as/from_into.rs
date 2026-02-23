use core::fmt::Display;

use crate::{
    Error,
    de::{CellDeserialize, CellDeserializeAs, CellParser, CellParserError},
    ser::{CellBuilder, CellBuilderError, CellSerialize, CellSerializeAs},
};

pub use crate::bits::{FromInto, FromIntoRef, TryFromInto, TryFromIntoRef};

impl<T, As> CellSerializeAs<T> for FromInto<As>
where
    T: Into<As> + Clone,
    As: CellSerialize,
{
    type Args = As::Args;

    #[inline]
    fn store_as(
        source: &T,
        builder: &mut CellBuilder,
        args: Self::Args,
    ) -> Result<(), CellBuilderError> {
        source.clone().into().store(builder, args)
    }
}

impl<'de, T, As> CellDeserializeAs<'de, T> for FromInto<As>
where
    As: Into<T> + CellDeserialize<'de>,
{
    type Args = As::Args;

    #[inline]
    fn parse_as(parser: &mut CellParser<'de>, args: Self::Args) -> Result<T, CellParserError<'de>> {
        As::parse(parser, args).map(Into::into)
    }
}

impl<T, As> CellSerializeAs<T> for FromIntoRef<As>
where
    for<'a> &'a T: Into<As>,
    As: CellSerialize,
{
    type Args = As::Args;

    #[inline]
    fn store_as(
        source: &T,
        builder: &mut CellBuilder,
        args: Self::Args,
    ) -> Result<(), CellBuilderError> {
        source.into().store(builder, args)
    }
}

impl<'de, T, As> CellDeserializeAs<'de, T> for FromIntoRef<As>
where
    As: Into<T> + CellDeserialize<'de>,
{
    type Args = As::Args;

    #[inline]
    fn parse_as(parser: &mut CellParser<'de>, args: Self::Args) -> Result<T, CellParserError<'de>> {
        As::parse(parser, args).map(Into::into)
    }
}

impl<T, As> CellSerializeAs<T> for TryFromInto<As>
where
    T: TryInto<As> + Clone,
    <T as TryInto<As>>::Error: Display,
    As: CellSerialize,
{
    type Args = As::Args;

    #[inline]
    fn store_as(
        source: &T,
        builder: &mut CellBuilder,
        args: Self::Args,
    ) -> Result<(), CellBuilderError> {
        source
            .clone()
            .try_into()
            .map_err(Error::custom)?
            .store(builder, args)
    }
}

impl<'de, T, As> CellDeserializeAs<'de, T> for TryFromInto<As>
where
    As: TryInto<T> + CellDeserialize<'de>,
    <As as TryInto<T>>::Error: Display,
{
    type Args = As::Args;

    #[inline]
    fn parse_as(parser: &mut CellParser<'de>, args: Self::Args) -> Result<T, CellParserError<'de>> {
        As::parse(parser, args)?.try_into().map_err(Error::custom)
    }
}

impl<T, As> CellSerializeAs<T> for TryFromIntoRef<As>
where
    for<'a> &'a T: TryInto<As> + Clone,
    for<'a> <&'a T as TryInto<As>>::Error: Display,
    As: CellSerialize,
{
    type Args = As::Args;

    #[inline]
    fn store_as(
        source: &T,
        builder: &mut CellBuilder,
        args: Self::Args,
    ) -> Result<(), CellBuilderError> {
        source
            .clone()
            .try_into()
            .map_err(Error::custom)?
            .store(builder, args)
    }
}

impl<'de, T, As> CellDeserializeAs<'de, T> for TryFromIntoRef<As>
where
    As: TryInto<T> + CellDeserialize<'de>,
    <As as TryInto<T>>::Error: Display,
{
    type Args = As::Args;

    #[inline]
    fn parse_as(parser: &mut CellParser<'de>, args: Self::Args) -> Result<T, CellParserError<'de>> {
        As::parse(parser, args)?.try_into().map_err(Error::custom)
    }
}
