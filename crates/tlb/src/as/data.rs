use core::marker::PhantomData;

use crate::{
    BitPackAs, BitReader, BitUnpackAs, BitWriter, CellBuilder, CellDeserializeAs, CellParser,
    CellSerializeAs, Same,
};

pub struct Data<As: ?Sized = Same>(PhantomData<As>);

impl<T, As> CellSerializeAs<T> for Data<As>
where
    As: BitPackAs<T> + ?Sized,
    T: ?Sized,
{
    #[inline]
    fn store_as(
        source: &T,
        builder: &mut CellBuilder,
    ) -> Result<(), <CellBuilder as BitWriter>::Error> {
        As::pack_as(source, builder)
    }
}

impl<'de, T, As> CellDeserializeAs<'de, T> for Data<As>
where
    As: BitUnpackAs<T> + ?Sized,
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de>) -> Result<T, <CellParser<'de> as BitReader>::Error> {
        As::unpack_as(parser)
    }
}
