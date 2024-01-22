use core::mem::MaybeUninit;
use std::{rc::Rc, sync::Arc};

use bitvec::{order::Msb0, slice::BitSlice};

use crate::{Cell, ErrorReason, Result, TLBDeserializeAs, TLBDeserializeAsWrap};

pub trait TLBDeserialize<'de>: Sized {
    fn parse(parser: &mut CellParser<'de>) -> Result<Self>;
}

pub trait TLBDeserializeOwned: for<'de> TLBDeserialize<'de> {}
impl<T> TLBDeserializeOwned for T where T: for<'de> TLBDeserialize<'de> {}

pub trait TLBDeserializeExt<'de>: TLBDeserialize<'de> {
    fn parse_fully(cell: &'de Cell) -> Result<Self> {
        cell.parser().parse_fully()
    }
}

impl<'de, T> TLBDeserializeExt<'de> for T where T: TLBDeserialize<'de> {}

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
        T: TLBDeserialize<'a>,
    {
        T::parse(self)
    }

    #[inline]
    pub fn parse_as<T, As>(&mut self) -> Result<T>
    where
        As: TLBDeserializeAs<'a, T>,
    {
        self.parse::<TLBDeserializeAsWrap<T, As>>()
            .map(TLBDeserializeAsWrap::into_inner)
    }

    #[inline]
    pub fn parse_fully<T>(&mut self) -> Result<T>
    where
        T: TLBDeserialize<'a>,
    {
        let v = self.parse()?;
        self.ensure_empty()?;
        Ok(v)
    }

    #[inline]
    pub fn parse_fully_as<T, As>(&mut self) -> Result<T>
    where
        As: TLBDeserializeAs<'a, T>,
    {
        self.parse_fully::<TLBDeserializeAsWrap<T, As>>()
            .map(TLBDeserializeAsWrap::into_inner)
    }

    #[inline]
    pub fn parse_reference<T>(&mut self) -> Result<T>
    where
        T: TLBDeserialize<'a>,
    {
        let reference = self.pop_reference()?;
        T::parse_fully(reference)
    }

    #[inline]
    pub fn parse_reference_as<T, As>(&mut self) -> Result<T>
    where
        As: TLBDeserializeAs<'a, T>,
    {
        self.parse_reference::<TLBDeserializeAsWrap<T, As>>()
            .map(TLBDeserializeAsWrap::into_inner)
    }

    #[inline]
    pub fn pop_reference(&mut self) -> Result<&'a Arc<Cell>> {
        let reference;
        (reference, self.references) = self
            .references
            .split_first()
            .ok_or(ErrorReason::NoMoreLeft)?;
        Ok(reference)
    }

    #[inline]
    pub fn load_bit(&mut self) -> Result<bool> {
        let bit;
        (bit, self.data) = self.data.split_first().ok_or(ErrorReason::NoMoreLeft)?;
        Ok(*bit)
    }

    #[inline]
    pub fn load_bits(&mut self, n: usize) -> Result<&'a BitSlice<u8, Msb0>> {
        if n > self.data.len() {
            return Err(ErrorReason::NoMoreLeft.into());
        }
        let loaded;
        (loaded, self.data) = self.data.split_at(n);
        Ok(loaded)
    }

    #[inline]
    pub fn load_bytes(&mut self, n: usize) -> Result<&'a BitSlice<u8, Msb0>> {
        self.load_bits(n * 8)
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty() && self.references.is_empty()
    }

    #[inline]
    pub fn ensure_empty(&self) -> Result<()> {
        if !self.is_empty() {
            Err(ErrorReason::MoreLeft.into())
        } else {
            Ok(())
        }
    }
}

impl<'de> TLBDeserialize<'de> for () {
    fn parse(_parser: &mut CellParser<'de>) -> Result<Self> {
        Ok(())
    }
}

impl<'de> TLBDeserialize<'de> for bool {
    fn parse(parser: &mut CellParser<'de>) -> Result<Self> {
        parser.load_bit()
    }
}

macro_rules! impl_tlb_deserialize_for_tuple {
    ($($n:tt:$t:ident),+) => {
        impl<'de, $($t),+> TLBDeserialize<'de> for ($($t,)+)
        where $(
            $t: TLBDeserialize<'de>,
        )+
        {
            #[inline]
            fn parse(parser: &mut CellParser<'de>) -> Result<Self> {
                Ok(($(
                    parser.parse::<$t>().map_err(|err| err.with_nth($n))?,
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

impl<'de, T, const N: usize> TLBDeserialize<'de> for [T; N]
where
    T: TLBDeserialize<'de>,
{
    fn parse(parser: &mut CellParser<'de>) -> Result<Self> {
        let mut arr: [MaybeUninit<T>; N] = unsafe { MaybeUninit::uninit().assume_init() };
        for a in &mut arr {
            a.write(parser.parse()?);
        }
        Ok(unsafe { arr.as_ptr().cast::<[T; N]>().read() })
    }
}

impl<'de, T> TLBDeserialize<'de> for Box<T>
where
    T: TLBDeserialize<'de>,
{
    fn parse(parser: &mut CellParser<'de>) -> Result<Self> {
        T::parse(parser).map(Box::new)
    }
}

impl<'de, T> TLBDeserialize<'de> for Rc<T>
where
    T: TLBDeserialize<'de>,
{
    fn parse(parser: &mut CellParser<'de>) -> Result<Self> {
        T::parse(parser).map(Rc::new)
    }
}

impl<'de, T> TLBDeserialize<'de> for Arc<T>
where
    T: TLBDeserialize<'de>,
{
    fn parse(parser: &mut CellParser<'de>) -> Result<Self> {
        T::parse(parser).map(Arc::new)
    }
}
