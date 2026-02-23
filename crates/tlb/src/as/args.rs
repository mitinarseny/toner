use crate::{
    de::{CellDeserializeAs, CellParser, CellParserError},
    ser::{CellBuilder, CellBuilderError, CellSerializeAs},
};

pub use crate::bits::DefaultArgs;

impl<T, As> CellSerializeAs<T> for DefaultArgs<As>
where
    As: CellSerializeAs<T>,
    As::Args: Default,
{
    type Args = ();

    #[inline]
    fn store_as(
        source: &T,
        builder: &mut CellBuilder,
        _: Self::Args,
    ) -> Result<(), CellBuilderError> {
        As::store_as(source, builder, <As::Args>::default())
    }
}

impl<'de, T, As> CellDeserializeAs<'de, T> for DefaultArgs<As>
where
    As: CellDeserializeAs<'de, T>,
    As::Args: Default,
{
    type Args = ();

    #[inline]
    fn parse_as(parser: &mut CellParser<'de>, _: Self::Args) -> Result<T, CellParserError<'de>> {
        As::parse_as(parser, <As::Args>::default())
    }
}
