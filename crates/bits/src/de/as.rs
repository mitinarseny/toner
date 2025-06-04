use core::mem::MaybeUninit;
use std::{borrow::Cow, rc::Rc, sync::Arc};

use bitvec::{order::Msb0, slice::BitSlice};
use either::Either;

use crate::{Context, Error, StringError, r#as::AsWrap};

use super::{BitReader, BitReaderExt, BitUnpack};

/// Adapter to **de**serialize `T`.  
/// See [`as`](crate::as) module-level documentation for more.
///
/// For dynamic arguments, see
/// [`BitUnackAsWithArgs`](super::args::as::BitUnpackAsWithArgs).
pub trait BitUnpackAs<'de, T> {
    /// Unpacks value using an adapter
    fn unpack_as<R>(reader: R) -> Result<T, R::Error>
    where
        R: BitReader<'de>;
}

/// **De**serialize value from [`BitSlice`] using an adapter
#[inline]
pub fn unpack_as<'de, T, As>(mut bits: &'de BitSlice<u8, Msb0>) -> Result<T, StringError>
where
    As: BitUnpackAs<'de, T>,
{
    bits.unpack_as::<T, As>()
}

/// **De**serialize value from bytes slice using an adapter
#[inline]
pub fn unpack_bytes_as<'de, T, As>(bytes: &'de [u8]) -> Result<T, StringError>
where
    As: BitUnpackAs<'de, T>,
{
    unpack_as::<_, As>(BitSlice::from_slice(bytes))
}

/// **De**serialize value from [`BitSlice`] using an adapter
/// and ensure that no more data left.
#[inline]
pub fn unpack_fully_as<'de, T, As>(mut bits: &'de BitSlice<u8, Msb0>) -> Result<T, StringError>
where
    As: BitUnpackAs<'de, T>,
{
    let v = bits.unpack_as::<T, As>()?;
    if !bits.is_empty() {
        return Err(Error::custom("more data left"));
    }
    Ok(v)
}

/// **De**serialize value from bytes slice using an adapter
/// and ensure that no more data left.
#[inline]
pub fn unpack_bytes_fully_as<'de, T, As>(bytes: &'de [u8]) -> Result<T, StringError>
where
    As: BitUnpackAs<'de, T>,
{
    unpack_fully_as::<_, As>(BitSlice::from_slice(bytes))
}

impl<'de, T, As, const N: usize> BitUnpackAs<'de, [T; N]> for [As; N]
where
    As: BitUnpackAs<'de, T>,
{
    #[inline]
    fn unpack_as<R>(mut reader: R) -> Result<[T; N], R::Error>
    where
        R: BitReader<'de>,
    {
        let mut arr: [MaybeUninit<T>; N] = unsafe { MaybeUninit::uninit().assume_init() };
        for a in &mut arr {
            a.write(reader.unpack_as::<T, As>()?);
        }
        Ok(unsafe { arr.as_ptr().cast::<[T; N]>().read() })
    }
}

macro_rules! impl_bit_unpack_as_for_tuple {
    ($($n:tt:$t:ident as $a:ident),+) => {
        impl<'de, $($t, $a),+> BitUnpackAs<'de, ($($t,)+)> for ($($a,)+)
        where $(
            $a: BitUnpackAs<'de, $t>,
        )+
        {
            #[inline]
            fn unpack_as<R>(mut reader: R) -> Result<($($t,)+), R::Error>
            where
                R: BitReader<'de>,
            {
                Ok(($(
                    $a::unpack_as(&mut reader)
                        .context(concat!(".", stringify!($n)))?,
                )+))
            }
        }
    };
}
impl_bit_unpack_as_for_tuple!(0:T0 as As0);
impl_bit_unpack_as_for_tuple!(0:T0 as As0,1:T1 as As1);
impl_bit_unpack_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2);
impl_bit_unpack_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3);
impl_bit_unpack_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4);
impl_bit_unpack_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5);
impl_bit_unpack_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5,6:T6 as As6);
impl_bit_unpack_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5,6:T6 as As6,7:T7 as As7);
impl_bit_unpack_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5,6:T6 as As6,7:T7 as As7,8:T8 as As8);
impl_bit_unpack_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5,6:T6 as As6,7:T7 as As7,8:T8 as As8,9:T9 as As9);

impl<'de, T, As> BitUnpackAs<'de, Box<T>> for Box<As>
where
    As: BitUnpackAs<'de, T> + ?Sized,
{
    #[inline]
    fn unpack_as<R>(reader: R) -> Result<Box<T>, R::Error>
    where
        R: BitReader<'de>,
    {
        AsWrap::<T, As>::unpack(reader)
            .map(AsWrap::into_inner)
            .map(Box::new)
    }
}

impl<'de, T, As> BitUnpackAs<'de, Rc<T>> for Rc<As>
where
    As: BitUnpackAs<'de, T> + ?Sized,
{
    #[inline]
    fn unpack_as<R>(reader: R) -> Result<Rc<T>, R::Error>
    where
        R: BitReader<'de>,
    {
        AsWrap::<T, As>::unpack(reader)
            .map(AsWrap::into_inner)
            .map(Rc::new)
    }
}

impl<'de, T, As> BitUnpackAs<'de, Arc<T>> for Arc<As>
where
    As: BitUnpackAs<'de, T> + ?Sized,
{
    #[inline]
    fn unpack_as<R>(reader: R) -> Result<Arc<T>, R::Error>
    where
        R: BitReader<'de>,
    {
        AsWrap::<T, As>::unpack(reader)
            .map(AsWrap::into_inner)
            .map(Arc::new)
    }
}

/// Always unpacks as [`Cow::Owned`]
impl<'de, 'a, T, As> BitUnpackAs<'de, Cow<'a, T>> for Cow<'a, As>
where
    T: ToOwned + ?Sized,
    As: ToOwned + ?Sized,
    As::Owned: BitUnpackAs<'de, T::Owned>,
{
    #[inline]
    fn unpack_as<R>(reader: R) -> Result<Cow<'a, T>, R::Error>
    where
        R: BitReader<'de>,
    {
        AsWrap::<T::Owned, As::Owned>::unpack(reader)
            .map(AsWrap::into_inner)
            .map(Cow::Owned)
    }
}

/// Implementation of [`Either X Y`](https://docs.ton.org/develop/data-formats/tl-b-types#either):
/// ```tlb
/// left$0 {X:Type} {Y:Type} value:X = Either X Y;
/// right$1 {X:Type} {Y:Type} value:Y = Either X Y;
/// ```
impl<'de, Left, Right, AsLeft, AsRight> BitUnpackAs<'de, Either<Left, Right>>
    for Either<AsLeft, AsRight>
where
    AsLeft: BitUnpackAs<'de, Left>,
    AsRight: BitUnpackAs<'de, Right>,
{
    #[inline]
    fn unpack_as<R>(reader: R) -> Result<Either<Left, Right>, R::Error>
    where
        R: BitReader<'de>,
    {
        Ok(
            Either::<AsWrap<Left, AsLeft>, AsWrap<Right, AsRight>>::unpack(reader)?
                .map_either(AsWrap::into_inner, AsWrap::into_inner),
        )
    }
}

impl<'de, T, As> BitUnpackAs<'de, Option<T>> for Either<(), As>
where
    As: BitUnpackAs<'de, T>,
{
    #[inline]
    fn unpack_as<R>(reader: R) -> Result<Option<T>, R::Error>
    where
        R: BitReader<'de>,
    {
        Ok(Either::<(), AsWrap<T, As>>::unpack(reader)?
            .map_right(AsWrap::into_inner)
            .right())
    }
}

/// Implementation of [`Maybe X`](https://docs.ton.org/develop/data-formats/tl-b-types#maybe):
/// ```tlb
/// nothing$0 {X:Type} = Maybe X;
/// just$1 {X:Type} value:X = Maybe X;
/// ```
impl<'de, T, As> BitUnpackAs<'de, Option<T>> for Option<As>
where
    As: BitUnpackAs<'de, T>,
{
    #[inline]
    fn unpack_as<R>(reader: R) -> Result<Option<T>, R::Error>
    where
        R: BitReader<'de>,
    {
        Ok(Option::<AsWrap<T, As>>::unpack(reader)?.map(AsWrap::into_inner))
    }
}
