use std::sync::Arc;

use bitvec::{mem::bits_of, order::Msb0, slice::BitSlice, vec::BitVec};

use crate::{Cell, Error, Result};

pub trait TLBDeserialize: Sized {
    fn parse(parser: &mut CellParser<'_>) -> Result<Self>;
}

pub trait TLBDeserializeExt: TLBDeserialize {
    fn parse_fully(cell: &Cell) -> Result<Self> {
        let mut parser = cell.parser();
        let v = parser.parse()?;
        parser.is_empty().then_some(v).ok_or(Error::MoreLeft)
    }
}

impl<T> TLBDeserializeExt for T where T: TLBDeserialize {}

pub struct CellParser<'a> {
    data: &'a BitSlice<u8, Msb0>,
    references: &'a [Arc<Cell>],
}

impl<'a> CellParser<'a> {
    pub(crate) const fn new(data: &'a BitSlice<u8, Msb0>, references: &'a [Arc<Cell>]) -> Self {
        Self { data, references }
    }

    #[inline]
    pub fn parse<T>(&mut self) -> Result<T>
    where
        T: TLBDeserialize,
    {
        T::parse(self)
    }

    #[inline]
    pub fn parse_reference<T>(&mut self) -> Result<T>
    where
        T: TLBDeserialize,
    {
        let reference = self.pop_reference().ok_or(Error::NoMoreLeft)?;
        T::parse_fully(reference)
    }

    #[inline]
    fn pop_reference(&mut self) -> Option<&Arc<Cell>> {
        let reference;
        (reference, self.references) = self.references.split_first()?;
        Some(reference)
    }

    #[inline]
    pub fn load_bit(&mut self) -> Result<bool> {
        let bit;
        (bit, self.data) = self.data.split_first().ok_or(Error::NoMoreLeft)?;
        Ok(*bit)
    }

    #[inline]
    pub fn load_bits(&mut self, n: usize) -> Result<&BitSlice<u8, Msb0>> {
        if n > self.data.len() {
            return Err(Error::NoMoreLeft);
        }
        let loaded;
        (loaded, self.data) = self.data.split_at(n);
        Ok(loaded)
    }

    #[inline]
    pub(crate) fn load_bitvec(&mut self, n: usize) -> Result<BitVec<u8, Msb0>> {
        Ok(self.load_bits(n)?.to_bitvec())
    }

    #[inline]
    pub fn load_bytes(&mut self, n: usize) -> Result<Vec<u8>> {
        Ok(self.load_bitvec(n * bits_of::<u8>())?.into_vec())
    }

    #[inline]
    pub fn load_bytes_array<const N: usize>(&mut self) -> Result<[u8; N]> {
        Ok(self
            .load_bitvec(N * bits_of::<u8>())?
            .as_raw_slice()
            .try_into()
            .unwrap())
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty() && self.references.is_empty()
    }
}

impl TLBDeserialize for () {
    fn parse(_parser: &mut CellParser<'_>) -> Result<Self> {
        Ok(())
    }
}

impl TLBDeserialize for bool {
    fn parse(parser: &mut CellParser<'_>) -> Result<Self> {
        parser.load_bit()
    }
}

macro_rules! impl_tlb_deserialize_for_tuple {
    ($($n:tt:$t:ident),+) => {
        impl<$($t),+> TLBDeserialize for ($($t,)+)
        where $(
            $t: TLBDeserialize,
        )+
        {
            #[inline]
            fn parse(parser: &mut CellParser<'_>) -> Result<Self> {
                Ok(($(
                    parser.parse::<$t>()?,
                )+))
            }
        }
    };
}
impl_tlb_deserialize_for_tuple!(0:T0);
impl_tlb_deserialize_for_tuple!(0:T0,1:T1);
impl_tlb_deserialize_for_tuple!(0:T0,1:T1,2:T2);
impl_tlb_deserialize_for_tuple!(0:T0,1:T1,2:T2,3:T3);
impl_tlb_deserialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4);
impl_tlb_deserialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5);
impl_tlb_deserialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6);
impl_tlb_deserialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6,7:T7);
impl_tlb_deserialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6,7:T7,8:T8);
impl_tlb_deserialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6,7:T7,8:T8,9:T9);

/// [Maybe](https://docs.ton.org/develop/data-formats/tl-b-types#maybe)
impl<T> TLBDeserialize for Option<T>
where
    T: TLBDeserialize,
{
    #[inline]
    fn parse(parser: &mut CellParser<'_>) -> Result<Self> {
        Ok(match parser.parse()? {
            false => None,
            true => Some(parser.parse()?),
        })
    }
}
