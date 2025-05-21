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

pub use crate::bits::r#as::Same;

impl<T> CellSerializeAs<T> for Same
where
    T: CellSerialize,
{
    #[inline]
    fn store_as(source: &T, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        source.store(builder)
    }
}

impl<T> CellSerializeAsWithArgs<T> for Same
where
    T: CellSerializeWithArgs,
{
    type Args = T::Args;

    #[inline]
    fn store_as_with(
        source: &T,
        builder: &mut CellBuilder,
        args: Self::Args,
    ) -> Result<(), CellBuilderError> {
        T::store_with(source, builder, args)
    }
}

impl<'de, T> CellDeserializeAs<'de, T> for Same
where
    T: CellDeserialize<'de>,
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de>) -> Result<T, CellParserError<'de>> {
        T::parse(parser)
    }
}

impl<'de, T> CellDeserializeAsWithArgs<'de, T> for Same
where
    T: CellDeserializeWithArgs<'de>,
{
    type Args = T::Args;

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de>,
        args: Self::Args,
    ) -> Result<T, CellParserError<'de>> {
        T::parse_with(parser, args)
    }
}
