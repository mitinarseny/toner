use crate::{
    de::{r#as::CellDeserializeAs, CellDeserialize, CellParser, CellParserError},
    ser::{r#as::CellSerializeAs, CellBuilder, CellBuilderError, CellSerialize},
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

impl<'de, T> CellDeserializeAs<'de, T> for Same
where
    T: CellDeserialize<'de>,
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de>) -> Result<T, CellParserError<'de>> {
        T::parse(parser)
    }
}
