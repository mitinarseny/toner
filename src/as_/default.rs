use core::marker::PhantomData;

use crate::{
    CellBuilder, CellParser, Result, Same, TLBDeserializeAs, TLBSerialize, TLBSerializeAs,
    TLBSerializeAsWrap,
};

pub struct DefaultOnNone<T = Same>(PhantomData<T>);

impl<T, U> TLBSerializeAs<T> for DefaultOnNone<U>
where
    U: TLBSerializeAs<T>,
{
    fn store_as(source: &T, builder: &mut CellBuilder) -> Result<()> {
        Some(TLBSerializeAsWrap::<T, U>::new(source)).store(builder)
    }
}

impl<'de, T, U> TLBDeserializeAs<'de, T> for DefaultOnNone<U>
where
    U: TLBDeserializeAs<'de, T>,
    T: Default,
{
    fn parse_as(parser: &mut CellParser<'de>) -> Result<T> {
        Option::<U>::parse_as(parser).map(Option::unwrap_or_default)
    }
}
