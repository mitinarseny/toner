use crate::{
    de::{
        CellParser, CellParserError, args::r#as::CellDeserializeAsWithArgs, r#as::CellDeserializeAs,
    },
    ser::{
        CellBuilder, CellBuilderError, args::r#as::CellSerializeAsWithArgs, r#as::CellSerializeAs,
    },
};

pub use crate::bits::r#as::args::{DefaultArgs, NoArgs};

impl<T, As, Args> CellSerializeAsWithArgs<T> for NoArgs<Args, As>
where
    As: CellSerializeAs<T> + ?Sized,
{
    type Args = Args;

    #[inline]
    fn store_as_with(
        source: &T,
        builder: &mut CellBuilder,
        _args: Self::Args,
    ) -> Result<(), CellBuilderError> {
        As::store_as(source, builder)
    }
}

impl<'de, T, As, Args> CellDeserializeAsWithArgs<'de, T> for NoArgs<Args, As>
where
    As: CellDeserializeAs<'de, T> + ?Sized,
{
    type Args = Args;

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de>,
        _args: Self::Args,
    ) -> Result<T, CellParserError<'de>> {
        As::parse_as(parser)
    }
}

impl<T, As> CellSerializeAs<T> for DefaultArgs<As>
where
    As: CellSerializeAsWithArgs<T>,
    As::Args: Default,
{
    #[inline]
    fn store_as(source: &T, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        As::store_as_with(source, builder, <As::Args>::default())
    }
}

impl<'de, T, As> CellDeserializeAs<'de, T> for DefaultArgs<As>
where
    As: CellDeserializeAsWithArgs<'de, T>,
    As::Args: Default,
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de>) -> Result<T, CellParserError<'de>> {
        As::parse_as_with(parser, <As::Args>::default())
    }
}
