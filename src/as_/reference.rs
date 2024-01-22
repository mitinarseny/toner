use core::marker::PhantomData;

use crate::{
    CellBuilder, CellParser, Result, Same, TLBDeserializeAs, TLBDeserializeAsWrap, TLBSerializeAs,
    TLBSerializeAsWrap,
};

pub struct Ref<T = Same>(PhantomData<T>);

impl<T, U> TLBSerializeAs<T> for Ref<U>
where
    U: TLBSerializeAs<T>,
    T: ?Sized,
{
    fn store_as(source: &T, builder: &mut CellBuilder) -> Result<()> {
        builder.store_reference(TLBSerializeAsWrap::<T, U>::new(source))?;
        Ok(())
    }
}

impl<'de, T, U> TLBDeserializeAs<'de, T> for Ref<U>
where
    U: TLBDeserializeAs<'de, T>,
{
    fn parse_as(parser: &mut CellParser<'de>) -> Result<T> {
        parser
            .parse_reference::<TLBDeserializeAsWrap<T, U>>()
            .map(TLBDeserializeAsWrap::into_inner)
    }
}
