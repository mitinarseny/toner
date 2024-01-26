use core::marker::PhantomData;

use crate::{
    BitReader, BitWriter, CellBuilder, CellDeserializeAs, CellParser, CellSerializeAs, Same,
};

pub struct Ref<As: ?Sized = Same>(PhantomData<As>);

impl<T, As> CellSerializeAs<T> for Ref<As>
where
    As: CellSerializeAs<T> + ?Sized,
{
    #[inline]
    fn store_as(
        source: &T,
        builder: &mut CellBuilder,
    ) -> Result<(), <CellBuilder as BitWriter>::Error> {
        builder.store_reference_as::<&T, &As>(source)?;
        Ok(())
    }
}

impl<'de, T, As> CellDeserializeAs<'de, T> for Ref<As>
where
    As: CellDeserializeAs<'de, T> + ?Sized,
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de>) -> Result<T, <CellParser<'de> as BitReader>::Error> {
        parser.parse_reference_as::<T, As>()
    }
}
