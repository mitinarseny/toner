use core::marker::PhantomData;

use tlbits::{de::args::r#as::BitUnpackAsWithArgs, ser::args::r#as::BitPackAsWithArgs};

use crate::{
    bits::{de::r#as::BitUnpackAs, ser::r#as::BitPackAs},
    de::{
        args::r#as::CellDeserializeAsWithArgs, r#as::CellDeserializeAs, CellParser, CellParserError,
    },
    ser::{
        args::r#as::CellSerializeAsWithArgs, r#as::CellSerializeAs, CellBuilder, CellBuilderError,
    },
};

use super::Same;

pub struct Data<As: ?Sized = Same>(PhantomData<As>);

impl<T, As> CellSerializeAs<T> for Data<As>
where
    As: BitPackAs<T> + ?Sized,
    T: ?Sized,
{
    #[inline]
    fn store_as(source: &T, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        As::pack_as(source, builder)
    }
}

impl<T, As> CellSerializeAsWithArgs<T> for Data<As>
where
    As: BitPackAsWithArgs<T> + ?Sized,
    T: ?Sized,
{
    type Args = As::Args;

    #[inline]
    fn store_as_with(
        source: &T,
        builder: &mut CellBuilder,
        args: Self::Args,
    ) -> Result<(), CellBuilderError> {
        As::pack_as_with(source, builder, args)
    }
}

impl<'de, T, As> CellDeserializeAs<'de, T> for Data<As>
where
    As: BitUnpackAs<T> + ?Sized,
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de>) -> Result<T, CellParserError<'de>> {
        As::unpack_as(parser)
    }
}

impl<'de, T, As> CellDeserializeAsWithArgs<'de, T> for Data<As>
where
    As: BitUnpackAsWithArgs<T> + ?Sized,
{
    type Args = As::Args;

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de>,
        args: Self::Args,
    ) -> Result<T, CellParserError<'de>> {
        As::unpack_as_with(parser, args)
    }
}
