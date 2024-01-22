use core::{fmt::Display, marker::PhantomData};

use crate::{
    CellBuilder, CellParser, ErrorReason, Result, TLBDeserialize, TLBDeserializeAs, TLBSerialize,
    TLBSerializeAs,
};

pub struct FromInto<T>(PhantomData<T>);

impl<T, U> TLBSerializeAs<T> for FromInto<U>
where
    T: Into<U> + Clone,
    U: TLBSerialize,
{
    fn store_as(source: &T, builder: &mut CellBuilder) -> Result<()> {
        source.clone().into().store(builder)
    }
}

impl<'de, T, U> TLBDeserializeAs<'de, T> for FromInto<U>
where
    U: Into<T> + TLBDeserialize<'de>,
{
    fn parse_as(parser: &mut crate::CellParser<'de>) -> Result<T> {
        U::parse(parser).map(Into::into)
    }
}

pub struct FromIntoRef<T>(PhantomData<T>);

impl<T, U> TLBSerializeAs<T> for FromIntoRef<U>
where
    for<'a> &'a T: Into<U>,
    U: TLBSerialize,
{
    fn store_as(source: &T, builder: &mut CellBuilder) -> Result<()> {
        source.into().store(builder)
    }
}

impl<'de, T, U> TLBDeserializeAs<'de, T> for FromIntoRef<U>
where
    U: Into<T> + TLBDeserialize<'de>,
{
    fn parse_as(parser: &mut crate::CellParser<'de>) -> Result<T> {
        U::parse(parser).map(Into::into)
    }
}

pub struct TryFromInto<T>(PhantomData<T>);

impl<T, U> TLBSerializeAs<T> for TryFromInto<U>
where
    T: TryInto<U> + Clone,
    <T as TryInto<U>>::Error: Display,
    U: TLBSerialize,
{
    fn store_as(source: &T, builder: &mut CellBuilder) -> Result<()> {
        source
            .clone()
            .try_into()
            .map_err(ErrorReason::custom)?
            .store(builder)
    }
}

impl<'de, T, U> TLBDeserializeAs<'de, T> for TryFromInto<U>
where
    U: TryInto<T> + TLBDeserialize<'de>,
    <U as TryInto<T>>::Error: Display,
{
    fn parse_as(parser: &mut CellParser<'de>) -> Result<T> {
        U::parse(parser)?
            .try_into()
            .map_err(ErrorReason::custom)
            .map_err(Into::into)
    }
}
