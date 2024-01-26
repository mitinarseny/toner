use core::{marker::PhantomData, mem::MaybeUninit};
use std::{rc::Rc, sync::Arc};

use tlbits::{BitReader, StringError};

use crate::{CellDeserialize, CellParser, ResultExt};

pub trait CellDeserializeAs<'de, T> {
    fn parse_as(parser: &mut CellParser<'de>) -> Result<T, <CellParser<'de> as BitReader>::Error>;
}

pub trait CellDeserializeAsOwned<T>: for<'de> CellDeserializeAs<'de, T> {}
impl<T, As> CellDeserializeAsOwned<As> for T where T: for<'de> CellDeserializeAs<'de, As> + ?Sized {}

pub struct CellDeserializeAsWrap<T, As>
where
    As: ?Sized,
{
    value: T,
    _phanton: PhantomData<As>,
}

impl<'de, T, As> CellDeserializeAsWrap<T, As>
where
    As: CellDeserializeAs<'de, T> + ?Sized,
{
    /// Return the inner value of type `T`.
    #[inline]
    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<'de, T, As> CellDeserialize<'de> for CellDeserializeAsWrap<T, As>
where
    As: CellDeserializeAs<'de, T> + ?Sized,
{
    #[inline]
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, StringError> {
        As::parse_as(parser).map(|value| Self {
            value,
            _phanton: PhantomData,
        })
    }
}

impl<'de, T, As, const N: usize> CellDeserializeAs<'de, [T; N]> for [As; N]
where
    As: CellDeserializeAs<'de, T>,
{
    #[inline]
    fn parse_as(
        parser: &mut CellParser<'de>,
    ) -> Result<[T; N], <CellParser<'de> as BitReader>::Error> {
        let mut arr: [MaybeUninit<T>; N] = unsafe { MaybeUninit::uninit().assume_init() };
        for a in &mut arr {
            a.write(parser.parse_as::<T, As>()?);
        }
        Ok(unsafe { arr.as_ptr().cast::<[T; N]>().read() })
    }
}

macro_rules! impl_cell_deserialize_as_for_tuple {
    ($($n:tt:$t:ident as $a:ident),+) => {
        impl<'de, $($t, $a),+> CellDeserializeAs<'de, ($($t,)+)> for ($($a,)+)
        where $(
            $a: CellDeserializeAs<'de, $t>,
        )+
        {
            #[inline]
            fn parse_as(parser: &mut CellParser<'de>) -> Result<($($t,)+), StringError> {
                Ok(($(
                    CellDeserializeAsWrap::<$t, $a>::parse(parser)
                        .context(concat!(".", stringify!($n)))?
                        .into_inner(),
                )+))
            }
        }
    };
}
impl_cell_deserialize_as_for_tuple!(0:T0 as As0);
impl_cell_deserialize_as_for_tuple!(0:T0 as As0,1:T1 as As1);
impl_cell_deserialize_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2);
impl_cell_deserialize_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3);
impl_cell_deserialize_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4);
impl_cell_deserialize_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5);
impl_cell_deserialize_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5,6:T6 as As6);
impl_cell_deserialize_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5,6:T6 as As6,7:T7 as As7);
impl_cell_deserialize_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5,6:T6 as As6,7:T7 as As7,8:T8 as As8);
impl_cell_deserialize_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5,6:T6 as As6,7:T7 as As7,8:T8 as As8,9:T9 as As9);

impl<'de, T, As> CellDeserializeAs<'de, Box<T>> for Box<As>
where
    As: CellDeserializeAs<'de, T> + ?Sized,
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de>) -> Result<Box<T>, StringError> {
        CellDeserializeAsWrap::<T, As>::parse(parser)
            .map(CellDeserializeAsWrap::into_inner)
            .map(Box::new)
    }
}

impl<'de, T, As> CellDeserializeAs<'de, Rc<T>> for Rc<As>
where
    As: CellDeserializeAs<'de, T> + ?Sized,
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de>) -> Result<Rc<T>, StringError> {
        CellDeserializeAsWrap::<T, As>::parse(parser)
            .map(CellDeserializeAsWrap::into_inner)
            .map(Rc::new)
    }
}

impl<'de, T, As> CellDeserializeAs<'de, Arc<T>> for Arc<As>
where
    As: CellDeserializeAs<'de, T> + ?Sized,
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de>) -> Result<Arc<T>, StringError> {
        CellDeserializeAsWrap::<T, As>::parse(parser)
            .map(CellDeserializeAsWrap::into_inner)
            .map(Arc::new)
    }
}

impl<'de, T, As> CellDeserializeAs<'de, Option<T>> for Option<As>
where
    As: CellDeserializeAs<'de, T>,
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de>) -> Result<Option<T>, StringError> {
        Ok(Option::<CellDeserializeAsWrap<T, As>>::parse(parser)?
            .map(CellDeserializeAsWrap::into_inner))
    }
}
