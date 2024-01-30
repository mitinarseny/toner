use crate::{
    CellBuilder, CellBuilderError, CellDeserialize, CellDeserializeAs, CellParser, CellParserError,
    CellSerialize, CellSerializeAs, Same,
};

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
