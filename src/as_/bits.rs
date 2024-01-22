use core::marker::PhantomData;

use bitvec::{
    order::{BitOrder, Msb0},
    slice::BitSlice,
    store::BitStore,
    view::AsBits,
};

use crate::{CellBuilder, CellParser, Result, TLBDeserializeAs, TLBSerialize, TLBSerializeAs};

pub struct AsBitSlice<S, O>(PhantomData<(S, O)>);

impl<T, S, O> TLBSerializeAs<T> for AsBitSlice<S, O>
where
    T: AsRef<BitSlice<S, O>> + ?Sized,
    S: BitStore,
    O: BitOrder,
{
    fn store_as(source: &T, builder: &mut CellBuilder) -> Result<()> {
        source.as_ref().store(builder)
    }
}

pub struct AsBytes;

impl<T> TLBSerializeAs<T> for AsBytes
where
    T: AsRef<[u8]> + ?Sized,
{
    fn store_as(source: &T, builder: &mut CellBuilder) -> Result<()> {
        source.as_bits::<Msb0>().store(builder)
    }
}

pub struct NBits<const BITS: usize>;

impl<const BITS: usize> NBits<BITS> {
    #[inline]
    pub fn load<'a>(parser: &mut CellParser<'a>) -> Result<&'a BitSlice<u8, Msb0>> {
        parser.load_bits(BITS)
    }
}

pub struct VarBytes<const BITS_FOR_BYTES_LEN: usize>;

impl<const BITS_FOR_BYTES_LEN: usize, T> TLBSerializeAs<T> for VarBytes<BITS_FOR_BYTES_LEN>
where
    T: AsRef<[u8]> + ?Sized,
{
    #[inline]
    fn store_as(source: &T, builder: &mut CellBuilder) -> Result<()> {
        let source = source.as_ref();
        builder
            .store_as::<_, NBits<BITS_FOR_BYTES_LEN>>(source.len())?
            .store_as::<_, AsBytes>(source)?;
        Ok(())
    }
}

impl<'de, const BITS_FOR_BYTES_LEN: usize> TLBDeserializeAs<'de, &'de BitSlice<u8, Msb0>>
    for VarBytes<BITS_FOR_BYTES_LEN>
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de>) -> Result<&'de BitSlice<u8, Msb0>> {
        let num_bytes = parser.parse_as::<_, NBits<BITS_FOR_BYTES_LEN>>()?;
        parser.load_bytes(num_bytes)
    }
}
