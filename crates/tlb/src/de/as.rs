use core::mem::MaybeUninit;
use std::{rc::Rc, sync::Arc};

use crate::{either::Either, r#as::AsWrap, ResultExt};

use super::{CellDeserialize, CellParser, CellParserError};

/// Adapter to **de**serialize `T`.  
/// See [`as`](crate::as) module-level documentation for more.
///
/// For dynamic arguments, see
/// [`CellDeserializeAsWithArgs`](super::args::as::CellDeserializeAsWithArgs).
pub trait CellDeserializeAs<'de, T> {
    fn parse_as(parser: &mut CellParser<'de>) -> Result<T, CellParserError<'de>>;
}

/// Owned version of [`CellDeserializeAs`]
pub trait CellDeserializeAsOwned<T>: for<'de> CellDeserializeAs<'de, T> {}
impl<T, As> CellDeserializeAsOwned<As> for T where T: for<'de> CellDeserializeAs<'de, As> + ?Sized {}

impl<'de, T, As, const N: usize> CellDeserializeAs<'de, [T; N]> for [As; N]
where
    As: CellDeserializeAs<'de, T>,
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de>) -> Result<[T; N], CellParserError<'de>> {
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
            fn parse_as(parser: &mut CellParser<'de>) -> Result<($($t,)+), CellParserError<'de>> {
                Ok(($(
                    AsWrap::<$t, $a>::parse(parser)
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
    fn parse_as(parser: &mut CellParser<'de>) -> Result<Box<T>, CellParserError<'de>> {
        AsWrap::<T, As>::parse(parser)
            .map(AsWrap::into_inner)
            .map(Box::new)
    }
}

impl<'de, T, As> CellDeserializeAs<'de, Rc<T>> for Rc<As>
where
    As: CellDeserializeAs<'de, T> + ?Sized,
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de>) -> Result<Rc<T>, CellParserError<'de>> {
        AsWrap::<T, As>::parse(parser)
            .map(AsWrap::into_inner)
            .map(Rc::new)
    }
}

impl<'de, T, As> CellDeserializeAs<'de, Arc<T>> for Arc<As>
where
    As: CellDeserializeAs<'de, T> + ?Sized,
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de>) -> Result<Arc<T>, CellParserError<'de>> {
        AsWrap::<T, As>::parse(parser)
            .map(AsWrap::into_inner)
            .map(Arc::new)
    }
}

/// Implementation of [`Either X Y`](https://docs.ton.org/develop/data-formats/tl-b-types#either):
/// ```tlb
/// left$0 {X:Type} {Y:Type} value:X = Either X Y;
/// right$1 {X:Type} {Y:Type} value:Y = Either X Y;
/// ```
impl<'de, Left, Right, AsLeft, AsRight> CellDeserializeAs<'de, Either<Left, Right>>
    for Either<AsLeft, AsRight>
where
    AsLeft: CellDeserializeAs<'de, Left>,
    AsRight: CellDeserializeAs<'de, Right>,
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de>) -> Result<Either<Left, Right>, CellParserError<'de>> {
        Ok(
            Either::<AsWrap<Left, AsLeft>, AsWrap<Right, AsRight>>::parse(parser)?
                .map_either(AsWrap::into_inner, AsWrap::into_inner),
        )
    }
}

impl<'de, T, As> CellDeserializeAs<'de, Option<T>> for Either<(), As>
where
    As: CellDeserializeAs<'de, T>,
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de>) -> Result<Option<T>, CellParserError<'de>> {
        Ok(Either::<(), AsWrap<T, As>>::parse(parser)?
            .map_right(AsWrap::into_inner)
            .right())
    }
}

/// Implementation of [`Maybe X`](https://docs.ton.org/develop/data-formats/tl-b-types#maybe):
/// ```tlb
/// nothing$0 {X:Type} = Maybe X;
/// just$1 {X:Type} value:X = Maybe X;
/// ```
impl<'de, T, As> CellDeserializeAs<'de, Option<T>> for Option<As>
where
    As: CellDeserializeAs<'de, T>,
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de>) -> Result<Option<T>, CellParserError<'de>> {
        Ok(Option::<AsWrap<T, As>>::parse(parser)?.map(AsWrap::into_inner))
    }
}
