use core::fmt::Display;

use crate::{
    de::{
        args::{r#as::CellDeserializeAsWithArgs, CellDeserializeWithArgs},
        r#as::CellDeserializeAs,
        CellDeserialize, CellParser, CellParserError,
    },
    ser::{
        args::{r#as::CellSerializeAsWithArgs, CellSerializeWithArgs},
        r#as::CellSerializeAs,
        CellBuilder, CellBuilderError, CellSerialize,
    },
    Error,
};

pub use crate::bits::r#as::{FromInto, FromIntoRef, TryFromInto, TryFromIntoRef};

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

impl<T, As> CellSerializeAsWithArgs<T> for FromInto<As>
where
    T: Into<As> + Clone,
    As: CellSerializeWithArgs,
{
    type Args = As::Args;

    #[inline]
    fn store_as_with(
        source: &T,
        builder: &mut CellBuilder,
        args: Self::Args,
    ) -> Result<(), CellBuilderError> {
        source.clone().into().store_with(builder, args)
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

impl<'de, T, As> CellDeserializeAsWithArgs<'de, T> for FromInto<As>
where
    As: Into<T> + CellDeserializeWithArgs<'de>,
{
    type Args = As::Args;

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de>,
        args: Self::Args,
    ) -> Result<T, CellParserError<'de>> {
        As::parse_with(parser, args).map(Into::into)
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

impl<T, As> CellSerializeAsWithArgs<T> for FromIntoRef<As>
where
    for<'a> &'a T: Into<As>,
    As: CellSerializeWithArgs,
{
    type Args = As::Args;

    #[inline]
    fn store_as_with(
        source: &T,
        builder: &mut CellBuilder,
        args: Self::Args,
    ) -> Result<(), CellBuilderError> {
        source.into().store_with(builder, args)
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

impl<'de, T, As> CellDeserializeAsWithArgs<'de, T> for FromIntoRef<As>
where
    As: Into<T> + CellDeserializeWithArgs<'de>,
{
    type Args = As::Args;

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de>,
        args: Self::Args,
    ) -> Result<T, CellParserError<'de>> {
        As::parse_with(parser, args).map(Into::into)
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

impl<T, As> CellSerializeAsWithArgs<T> for TryFromInto<As>
where
    T: TryInto<As> + Clone,
    <T as TryInto<As>>::Error: Display,
    As: CellSerializeWithArgs,
{
    type Args = As::Args;

    #[inline]
    fn store_as_with(
        source: &T,
        builder: &mut CellBuilder,
        args: Self::Args,
    ) -> Result<(), CellBuilderError> {
        source
            .clone()
            .try_into()
            .map_err(Error::custom)?
            .store_with(builder, args)
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

impl<'de, T, As> CellDeserializeAsWithArgs<'de, T> for TryFromInto<As>
where
    As: TryInto<T> + CellDeserializeWithArgs<'de>,
    <As as TryInto<T>>::Error: Display,
{
    type Args = As::Args;

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de>,
        args: Self::Args,
    ) -> Result<T, CellParserError<'de>> {
        As::parse_with(parser, args)?
            .try_into()
            .map_err(Error::custom)
    }
}

impl<T, As> CellSerializeAs<T> for TryFromIntoRef<As>
where
    for<'a> &'a T: TryInto<As>,
    for<'a> <&'a T as TryInto<As>>::Error: Display,
    As: CellSerialize,
{
    #[inline]
    fn store_as(source: &T, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        source.try_into().map_err(Error::custom)?.store(builder)
    }
}

impl<T, As> CellSerializeAsWithArgs<T> for TryFromIntoRef<As>
where
    for<'a> &'a T: TryInto<As> + Clone,
    for<'a> <&'a T as TryInto<As>>::Error: Display,
    As: CellSerializeWithArgs,
{
    type Args = As::Args;

    #[inline]
    fn store_as_with(
        source: &T,
        builder: &mut CellBuilder,
        args: Self::Args,
    ) -> Result<(), CellBuilderError> {
        source
            .clone()
            .try_into()
            .map_err(Error::custom)?
            .store_with(builder, args)
    }
}

impl<'de, T, As> CellDeserializeAs<'de, T> for TryFromIntoRef<As>
where
    As: TryInto<T> + CellDeserialize<'de>,
    <As as TryInto<T>>::Error: Display,
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de>) -> Result<T, CellParserError<'de>> {
        As::parse(parser)?.try_into().map_err(Error::custom)
    }
}

impl<'de, T, As> CellDeserializeAsWithArgs<'de, T> for TryFromIntoRef<As>
where
    As: TryInto<T> + CellDeserializeWithArgs<'de>,
    <As as TryInto<T>>::Error: Display,
{
    type Args = As::Args;

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de>,
        args: Self::Args,
    ) -> Result<T, CellParserError<'de>> {
        As::parse_with(parser, args)?
            .try_into()
            .map_err(Error::custom)
    }
}
