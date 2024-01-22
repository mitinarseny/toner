use crate::{
    CellBuilder, CellParser, Result, TLBDeserialize, TLBDeserializeAs, TLBSerialize, TLBSerializeAs,
};

pub struct Same;

impl<T> TLBSerializeAs<T> for Same
where
    T: TLBSerialize,
{
    fn store_as(source: &T, builder: &mut CellBuilder) -> Result<()> {
        source.store(builder)
    }
}

impl<'de, T> TLBDeserializeAs<'de, T> for Same
where
    T: TLBDeserialize<'de>,
{
    fn parse_as(parser: &mut CellParser<'de>) -> Result<T> {
        T::parse(parser)
    }
}
