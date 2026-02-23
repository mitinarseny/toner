use crate::{
    de::{CellDeserializeAs, CellParser, CellParserError},
    ser::{CellBuilder, CellBuilderError, CellSerializeAs},
};

pub use crate::bits::DefaultOnNone;

impl<T, As> CellSerializeAs<T> for DefaultOnNone<As>
where
    As: CellSerializeAs<T>,
    T: Default + PartialEq,
{
    type Args = As::Args;

    fn store_as(
        source: &T,
        builder: &mut CellBuilder,
        args: As::Args,
    ) -> Result<(), CellBuilderError> {
        builder.store_as::<_, Option<&As>>((source != &T::default()).then_some(source), args)?;
        Ok(())
    }
}

impl<'de, T, As> CellDeserializeAs<'de, T> for DefaultOnNone<As>
where
    As: CellDeserializeAs<'de, T>,
    T: Default,
{
    type Args = As::Args;

    #[inline]
    fn parse_as(parser: &mut CellParser<'de>, args: As::Args) -> Result<T, CellParserError<'de>> {
        parser
            .parse_as::<_, Option<As>>(args)
            .map(Option::unwrap_or_default)
    }
}
