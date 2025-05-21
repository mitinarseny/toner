//! **De**/**ser**ialization helpers for
//! [TL-B](https://docs.ton.org/develop/data-formats/tl-b-language).
//!
//! This approach is heavily inspired by
//! [serde_with](https://docs.rs/serde_with/latest/serde_with).
//! Please, read their docs for more usage examples.
mod args;
pub mod bin_tree;
mod data;
mod default;
mod from_into;
mod fully;
pub mod hashmap;
mod list;
mod reference;
mod same;

pub use self::{
    args::*, data::*, default::*, from_into::*, fully::*, list::*, reference::*, same::*,
};

use crate::{
    de::{
        CellDeserialize, CellParser, CellParserError,
        args::{CellDeserializeWithArgs, r#as::CellDeserializeAsWithArgs},
        r#as::CellDeserializeAs,
    },
    ser::{
        CellBuilder, CellBuilderError, CellSerialize,
        args::{CellSerializeWithArgs, r#as::CellSerializeAsWithArgs},
        r#as::CellSerializeAs,
    },
};

pub use tlbits::r#as::AsWrap;

impl<T, As> CellSerialize for AsWrap<&T, As>
where
    T: ?Sized,
    As: ?Sized,
    As: CellSerializeAs<T>,
{
    #[inline]
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        As::store_as(self.into_inner(), builder)
    }
}

impl<T, As> CellSerializeWithArgs for AsWrap<&T, As>
where
    T: ?Sized,
    As: CellSerializeAsWithArgs<T> + ?Sized,
{
    type Args = As::Args;

    #[inline]
    fn store_with(
        &self,
        builder: &mut CellBuilder,
        args: Self::Args,
    ) -> Result<(), CellBuilderError> {
        As::store_as_with(self.into_inner(), builder, args)
    }
}

impl<'de, T, As> CellDeserialize<'de> for AsWrap<T, As>
where
    As: CellDeserializeAs<'de, T> + ?Sized,
{
    #[inline]
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, CellParserError<'de>> {
        As::parse_as(parser).map(Self::new)
    }
}

impl<'de, T, As> CellDeserializeWithArgs<'de> for AsWrap<T, As>
where
    As: CellDeserializeAsWithArgs<'de, T> + ?Sized,
{
    type Args = As::Args;

    fn parse_with(
        parser: &mut CellParser<'de>,
        args: Self::Args,
    ) -> Result<Self, CellParserError<'de>> {
        As::parse_as_with(parser, args).map(Self::new)
    }
}
