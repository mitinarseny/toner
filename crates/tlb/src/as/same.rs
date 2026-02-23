use crate::{
    de::{CellDeserialize, CellDeserializeAs, CellParser, CellParserError},
    ser::{CellBuilder, CellBuilderError, CellSerialize, CellSerializeAs},
};

pub use crate::bits::Same;

impl<T> CellSerializeAs<T> for Same
where
    T: CellSerialize,
{
    type Args = T::Args;

    #[inline]
    fn store_as(
        source: &T,
        builder: &mut CellBuilder,
        args: Self::Args,
    ) -> Result<(), CellBuilderError> {
        T::store(source, builder, args)
    }
}

impl<'de, T> CellDeserializeAs<'de, T> for Same
where
    T: CellDeserialize<'de>,
{
    type Args = T::Args;

    #[inline]
    fn parse_as(parser: &mut CellParser<'de>, args: Self::Args) -> Result<T, CellParserError<'de>> {
        T::parse(parser, args)
    }
}
