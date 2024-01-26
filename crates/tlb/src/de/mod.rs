mod r#as;

pub use self::r#as::*;

use core::mem::MaybeUninit;
use std::sync::Arc;

use bitvec::{order::Msb0, slice::BitSlice};
use tlbits::BitReader;

use crate::{Cell, Error, ResultExt};

pub trait CellDeserialize<'de>: Sized {
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, <CellParser<'de> as BitReader>::Error>;
}

pub trait CellDeserializeOwned: for<'de> CellDeserialize<'de> {}
impl<T> CellDeserializeOwned for T where T: for<'de> CellDeserialize<'de> {}

impl<'de> CellDeserialize<'de> for () {
    #[inline]
    fn parse(_parser: &mut CellParser<'de>) -> Result<Self, <CellParser<'de> as BitReader>::Error> {
        Ok(())
    }
}

impl<'de, T, const N: usize> CellDeserialize<'de> for [T; N]
where
    T: CellDeserialize<'de>,
{
    #[inline]
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, <CellParser<'de> as BitReader>::Error> {
        let mut arr: [MaybeUninit<T>; N] = unsafe { MaybeUninit::uninit().assume_init() };
        for a in &mut arr {
            a.write(T::parse(parser)?);
        }
        Ok(unsafe { arr.as_ptr().cast::<[T; N]>().read() })
    }
}

macro_rules! impl_cell_deserialize_for_tuple {
    ($($n:tt:$t:ident),+) => {
        impl<'de, $($t),+> CellDeserialize<'de> for ($($t,)+)
        where $(
            $t: CellDeserialize<'de>,
        )+
        {
            #[inline]
            fn parse(parser: &mut CellParser<'de>) -> Result<Self, <CellParser<'de> as BitReader>::Error>
            {
                Ok(($(
                    $t::parse(parser).context(concat!(".", stringify!($n)))?,
                )+))
            }
        }
    };
}
impl_cell_deserialize_for_tuple!(0:T0);
impl_cell_deserialize_for_tuple!(0:T0,1:T1);
impl_cell_deserialize_for_tuple!(0:T0,1:T1,2:T2);
impl_cell_deserialize_for_tuple!(0:T0,1:T1,2:T2,3:T3);
impl_cell_deserialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4);
impl_cell_deserialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5);
impl_cell_deserialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6);
impl_cell_deserialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6,7:T7);
impl_cell_deserialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6,7:T7,8:T8);
impl_cell_deserialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6,7:T7,8:T8,9:T9);

pub struct CellParser<'de> {
    data: &'de BitSlice<u8, Msb0>,
    references: &'de [Arc<Cell>],
}

impl<'de> CellParser<'de> {
    #[inline]
    pub(crate) const fn new(data: &'de BitSlice<u8, Msb0>, references: &'de [Arc<Cell>]) -> Self {
        Self { data, references }
    }

    #[inline]
    pub fn parse<T>(&mut self) -> Result<T, <Self as BitReader>::Error>
    where
        T: CellDeserialize<'de>,
    {
        T::parse(self)
    }

    #[inline]
    pub fn parse_as<T, As>(&mut self) -> Result<T, <Self as BitReader>::Error>
    where
        As: CellDeserializeAs<'de, T> + ?Sized,
    {
        As::parse_as(self)
    }

    #[inline]
    fn pop_reference(&mut self) -> Result<&'de Arc<Cell>, <Self as BitReader>::Error> {
        let (first, rest) = self
            .references
            .split_first()
            .ok_or_else(|| Error::custom("no more references left"))?;
        self.references = rest;
        Ok(first)
    }

    #[inline]
    pub(crate) fn parse_reference_as<T, As>(&mut self) -> Result<T, <Self as BitReader>::Error>
    where
        As: CellDeserializeAs<'de, T> + ?Sized,
    {
        self.pop_reference()?.parse_fully_as::<T, As>()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty() && self.references.is_empty()
    }

    #[inline]
    pub fn ensure_empty(&self) -> Result<(), <Self as BitReader>::Error> {
        if !self.is_empty() {
            return Err(Error::custom(format!(
                "more data left: {} bits, {} references",
                self.data.len(),
                self.references.len(),
            )));
        }
        Ok(())
    }
}

impl<'de> BitReader for CellParser<'de> {
    type Error = <&'de BitSlice<u8, Msb0> as BitReader>::Error;

    #[inline]
    fn read_bit(&mut self) -> Result<bool, Self::Error> {
        self.data.read_bit()
    }

    #[inline]
    fn read_bits_into(&mut self, dst: &mut BitSlice<u8, Msb0>) -> Result<(), Self::Error> {
        self.data.read_bits_into(dst)
    }

    #[inline]
    fn skip(&mut self, n: usize) -> Result<(), Self::Error> {
        self.data.skip(n)
    }
}
