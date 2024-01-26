use crate::{
    BitReader, BitWriter, CellBuilder, CellDeserialize, CellDeserializeAs, CellParser,
    CellSerialize, CellSerializeAs, Same,
};

impl<T> CellSerializeAs<T> for Same
where
    T: CellSerialize,
{
    #[inline]
    fn store_as(
        source: &T,
        builder: &mut CellBuilder,
    ) -> Result<(), <CellBuilder as BitWriter>::Error> {
        source.store(builder)
    }
}

impl<'de, T> CellDeserializeAs<'de, T> for Same
where
    T: CellDeserialize<'de>,
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de>) -> Result<T, <CellParser<'de> as BitReader>::Error> {
        T::parse(parser)
    }
}
