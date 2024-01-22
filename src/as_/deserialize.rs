use core::marker::PhantomData;
use std::{rc::Rc, sync::Arc};

use crate::{CellParser, Result, TLBDeserialize};

pub trait TLBDeserializeAs<'de, T> {
    fn parse_as(parser: &mut CellParser<'de>) -> Result<T>;
}

pub struct TLBDeserializeAsWrap<T, U>
where
    U: ?Sized,
{
    value: T,
    _phanton: PhantomData<U>,
}

impl<'de, T, U> TLBDeserializeAsWrap<T, U>
where
    U: TLBDeserializeAs<'de, T> + ?Sized,
{
    /// Return the inner value of type `T`.
    #[inline]
    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<'de, T, U> TLBDeserialize<'de> for TLBDeserializeAsWrap<T, U>
where
    U: TLBDeserializeAs<'de, T> + ?Sized,
{
    #[inline]
    fn parse(parser: &mut CellParser<'de>) -> Result<Self> {
        U::parse_as(parser).map(|value| Self {
            value,
            _phanton: PhantomData,
        })
    }
}

macro_rules! impl_tlb_deserialize_as_for_tuple {
    ($($n:tt:$t:ident as $a:ident),+) => {
        impl<'de, $($t, $a),+> TLBDeserializeAs<'de, ($($t,)+)> for ($($a,)+)
        where $(
            $a: TLBDeserializeAs<'de, $t>,
        )+
        {
            #[inline]
            fn parse_as(parser: &mut CellParser<'de>) -> Result<($($t,)+)> {
                Ok(($(
                    TLBDeserializeAsWrap::<$t, $a>::parse(parser)
                        .map_err(|err| err.with_nth($n))?
                        .into_inner(),
                )+))
            }
        }
    };
}
impl_tlb_deserialize_as_for_tuple!(0:T0 as As0);
impl_tlb_deserialize_as_for_tuple!(0:T0 as As0,1:T1 as As1);
impl_tlb_deserialize_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2);
impl_tlb_deserialize_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3);
impl_tlb_deserialize_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4);
impl_tlb_deserialize_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5);
impl_tlb_deserialize_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5,6:T6 as As6);
impl_tlb_deserialize_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5,6:T6 as As6,7:T7 as As7);
impl_tlb_deserialize_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5,6:T6 as As6,7:T7 as As7,8:T8 as As8);
impl_tlb_deserialize_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5,6:T6 as As6,7:T7 as As7,8:T8 as As8,9:T9 as As9);

impl<'de, T, U> TLBDeserializeAs<'de, Box<T>> for Box<U>
where
    U: TLBDeserializeAs<'de, T> + ?Sized,
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de>) -> Result<Box<T>> {
        TLBDeserializeAsWrap::<T, U>::parse(parser)
            .map(TLBDeserializeAsWrap::into_inner)
            .map(Box::new)
    }
}

impl<'de, T, U> TLBDeserializeAs<'de, Rc<T>> for Rc<U>
where
    U: TLBDeserializeAs<'de, T> + ?Sized,
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de>) -> Result<Rc<T>> {
        TLBDeserializeAsWrap::<T, U>::parse(parser)
            .map(TLBDeserializeAsWrap::into_inner)
            .map(Rc::new)
    }
}

impl<'de, T, U> TLBDeserializeAs<'de, Arc<T>> for Arc<U>
where
    U: TLBDeserializeAs<'de, T> + ?Sized,
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de>) -> Result<Arc<T>> {
        TLBDeserializeAsWrap::<T, U>::parse(parser)
            .map(TLBDeserializeAsWrap::into_inner)
            .map(Arc::new)
    }
}

impl<'de, T, U> TLBDeserializeAs<'de, Option<T>> for Option<U>
where
    U: TLBDeserializeAs<'de, T>,
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de>) -> Result<Option<T>> {
        Ok(Option::<TLBDeserializeAsWrap<T, U>>::parse(parser)?
            .map(TLBDeserializeAsWrap::into_inner))
    }
}
