use core::marker::PhantomData;
use std::{rc::Rc, sync::Arc};

use crate::{CellBuilder, Result, TLBSerialize};

pub trait TLBSerializeAs<T: ?Sized> {
    fn store_as(source: &T, builder: &mut CellBuilder) -> Result<()>;
}

pub trait TLBSerializeWrapAs {
    fn wrap_as<T>(&self) -> TLBSerializeAsWrap<'_, Self, T>
    where
        T: TLBSerializeAs<Self>,
    {
        TLBSerializeAsWrap::new(self)
    }
}

impl<T> TLBSerializeWrapAs for T {}

pub struct TLBSerializeAsWrap<'a, T, U>
where
    U: TLBSerializeAs<T> + ?Sized,
    T: ?Sized,
{
    value: &'a T,
    _phantom: PhantomData<U>,
}

impl<'a, T, U> TLBSerializeAsWrap<'a, T, U>
where
    T: ?Sized,
    U: TLBSerializeAs<T> + ?Sized,
{
    #[inline]
    pub const fn new(value: &'a T) -> Self {
        Self {
            value,
            _phantom: PhantomData,
        }
    }
}

impl<'a, T, U> TLBSerialize for TLBSerializeAsWrap<'a, T, U>
where
    T: ?Sized,
    U: ?Sized,
    U: TLBSerializeAs<T>,
{
    #[inline]
    fn store(&self, builder: &mut CellBuilder) -> Result<()> {
        U::store_as(self.value, builder)
    }
}

impl<'a, T, U> TLBSerializeAs<&'a T> for &'a U
where
    U: TLBSerializeAs<T> + ?Sized,
    T: ?Sized,
{
    #[inline]
    fn store_as(source: &&'a T, builder: &mut CellBuilder) -> Result<()> {
        TLBSerializeAsWrap::<T, U>::new(source).store(builder)
    }
}

impl<'a, T, U> TLBSerializeAs<&'a mut T> for &'a mut U
where
    U: TLBSerializeAs<T> + ?Sized,
    T: ?Sized,
{
    #[inline]
    fn store_as(source: &&'a mut T, builder: &mut CellBuilder) -> Result<()> {
        TLBSerializeAsWrap::<T, U>::new(source).store(builder)
    }
}

macro_rules! impl_tlb_serialize_as_for_tuple {
    ($($n:tt:$t:ident as $a:ident),+) => {
        impl<$($t, $a),+> TLBSerializeAs<($($t,)+)> for ($($a,)+)
        where $(
            $a: TLBSerializeAs<$t>,
        )+
        {
            #[inline]
            fn store_as(source: &($($t,)+), builder: &mut CellBuilder) -> Result<()> {
                builder$(
                    .store(TLBSerializeAsWrap::<$t, $a>::new(&source.$n))
                    .map_err(|err| err.with_nth($n))?)+;
                Ok(())
            }
        }
    };
}
impl_tlb_serialize_as_for_tuple!(0:T0 as As0);
impl_tlb_serialize_as_for_tuple!(0:T0 as As0,1:T1 as As1);
impl_tlb_serialize_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2);
impl_tlb_serialize_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3);
impl_tlb_serialize_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4);
impl_tlb_serialize_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5);
impl_tlb_serialize_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5,6:T6 as As6);
impl_tlb_serialize_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5,6:T6 as As6,7:T7 as As7);
impl_tlb_serialize_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5,6:T6 as As6,7:T7 as As7,8:T8 as As8);
impl_tlb_serialize_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5,6:T6 as As6,7:T7 as As7,8:T8 as As8,9:T9 as As9);

impl<T, U> TLBSerializeAs<Box<T>> for Box<U>
where
    U: TLBSerializeAs<T> + ?Sized,
    T: ?Sized,
{
    #[inline]
    fn store_as(source: &Box<T>, builder: &mut CellBuilder) -> Result<()> {
        TLBSerializeAsWrap::<T, U>::new(source).store(builder)
    }
}

impl<T, U> TLBSerializeAs<Rc<T>> for Rc<U>
where
    U: TLBSerializeAs<T> + ?Sized,
    T: ?Sized,
{
    #[inline]
    fn store_as(source: &Rc<T>, builder: &mut CellBuilder) -> Result<()> {
        TLBSerializeAsWrap::<T, U>::new(source).store(builder)
    }
}

impl<T, U> TLBSerializeAs<Arc<T>> for Arc<U>
where
    U: TLBSerializeAs<T> + ?Sized,
    T: ?Sized,
{
    #[inline]
    fn store_as(source: &Arc<T>, builder: &mut CellBuilder) -> Result<()> {
        TLBSerializeAsWrap::<T, U>::new(source).store(builder)
    }
}

impl<T, U> TLBSerializeAs<Option<T>> for Option<U>
where
    U: TLBSerializeAs<T>,
{
    #[inline]
    fn store_as(source: &Option<T>, builder: &mut CellBuilder) -> Result<()> {
        source
            .as_ref()
            .map(TLBSerializeAsWrap::<T, U>::new)
            .store(builder)
    }
}

impl<T, As, const N: usize> TLBSerializeAs<[T; N]> for [As; N]
where
    As: TLBSerializeAs<T>,
{
    #[inline]
    fn store_as(source: &[T; N], builder: &mut CellBuilder) -> Result<()> {
        for (i, v) in source.iter().enumerate() {
            TLBSerializeAsWrap::<T, As>::new(v)
                .store(builder)
                .map_err(|err| err.with_nth(i))?;
        }
        Ok(())
    }
}

impl<T, As> TLBSerializeAs<[T]> for [As]
where
    As: TLBSerializeAs<T>,
{
    #[inline]
    fn store_as(source: &[T], builder: &mut CellBuilder) -> Result<()> {
        for (i, v) in source.iter().enumerate() {
            TLBSerializeAsWrap::<T, As>::new(v)
                .store(builder)
                .map_err(|err| err.with_nth(i))?;
        }
        Ok(())
    }
}
