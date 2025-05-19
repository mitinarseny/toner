use core::mem::MaybeUninit;
use std::{rc::Rc, sync::Arc};

use bitvec::{order::Msb0, slice::BitSlice};
use either::Either;

use super::{
    super::{BitReader, BitReaderExt},
    BitUnpackWithArgs,
};

use crate::{
    ResultExt, StringError,
    r#as::{AsWrap, args::NoArgs},
};

/// Adapter to **de**serialize `T` with args.  
/// See [`as`](crate::as) module-level documentation for more.
///
/// For version without arguments, see [`BitUnpackAs`](super::super::as::BitUnpackAs).
pub trait BitUnpackAsWithArgs<T> {
    type Args;

    /// Unpacks value with args using an adapter
    fn unpack_as_with<R>(reader: R, args: Self::Args) -> Result<T, R::Error>
    where
        R: BitReader;
}

/// **De**serialize value from [`BitSlice`] with args using an adapter
#[inline]
pub fn unpack_as_with<T, As>(
    bits: impl AsRef<BitSlice<u8, Msb0>>,
    args: As::Args,
) -> Result<T, StringError>
where
    As: BitUnpackAsWithArgs<T>,
{
    bits.as_ref().unpack_as_with::<_, As>(args)
}

impl<T, As, const N: usize> BitUnpackAsWithArgs<[T; N]> for [As; N]
where
    As: BitUnpackAsWithArgs<T>,
    As::Args: Clone,
{
    type Args = As::Args;

    #[inline]
    fn unpack_as_with<R>(mut reader: R, args: Self::Args) -> Result<[T; N], R::Error>
    where
        R: BitReader,
    {
        let mut arr: [MaybeUninit<T>; N] = unsafe { MaybeUninit::uninit().assume_init() };
        for a in &mut arr {
            a.write(reader.unpack_as_with::<T, As>(args.clone())?);
        }
        Ok(unsafe { arr.as_ptr().cast::<[T; N]>().read() })
    }
}

impl<T, As> BitUnpackAsWithArgs<Vec<T>> for Vec<As>
where
    As: BitUnpackAsWithArgs<T>,
    As::Args: Clone,
{
    type Args = (usize, As::Args);

    #[inline]
    fn unpack_as_with<R>(mut reader: R, (len, args): Self::Args) -> Result<Vec<T>, R::Error>
    where
        R: BitReader,
    {
        reader
            .unpack_iter_as_with::<_, As>(args)
            .take(len)
            .collect()
    }
}

macro_rules! impl_bit_unpack_as_with_args_for_tuple {
    ($($n:tt:$t:ident as $a:ident),+) => {
        impl<$($t, $a),+> BitUnpackAsWithArgs<($($t,)+)> for ($($a,)+)
        where $(
            $a: BitUnpackAsWithArgs<$t>,
        )+
        {
            type Args = ($($a::Args,)+);

            #[inline]
            fn unpack_as_with<R>(mut reader: R, args: Self::Args) -> Result<($($t,)+), R::Error>
            where
                R: BitReader,
            {
                Ok(($(
                    $a::unpack_as_with(&mut reader, args.$n)
                        .context(concat!(".", stringify!($n)))?,
                )+))
            }
        }
    };
}
impl_bit_unpack_as_with_args_for_tuple!(0:T0 as As0);
impl_bit_unpack_as_with_args_for_tuple!(0:T0 as As0,1:T1 as As1);
impl_bit_unpack_as_with_args_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2);
impl_bit_unpack_as_with_args_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3);
impl_bit_unpack_as_with_args_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4);
impl_bit_unpack_as_with_args_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5);
impl_bit_unpack_as_with_args_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5,6:T6 as As6);
impl_bit_unpack_as_with_args_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5,6:T6 as As6,7:T7 as As7);
impl_bit_unpack_as_with_args_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5,6:T6 as As6,7:T7 as As7,8:T8 as As8);
impl_bit_unpack_as_with_args_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5,6:T6 as As6,7:T7 as As7,8:T8 as As8,9:T9 as As9);

impl<T, As> BitUnpackAsWithArgs<Box<T>> for Box<As>
where
    As: BitUnpackAsWithArgs<T> + ?Sized,
{
    type Args = As::Args;

    #[inline]
    fn unpack_as_with<R>(reader: R, args: Self::Args) -> Result<Box<T>, R::Error>
    where
        R: BitReader,
    {
        AsWrap::<T, As>::unpack_with(reader, args)
            .map(AsWrap::into_inner)
            .map(Into::into)
    }
}

impl<T, As> BitUnpackAsWithArgs<Rc<T>> for Rc<As>
where
    As: BitUnpackAsWithArgs<T> + ?Sized,
{
    type Args = As::Args;

    #[inline]
    fn unpack_as_with<R>(reader: R, args: Self::Args) -> Result<Rc<T>, R::Error>
    where
        R: BitReader,
    {
        AsWrap::<T, As>::unpack_with(reader, args)
            .map(AsWrap::into_inner)
            .map(Into::into)
    }
}

impl<T, As> BitUnpackAsWithArgs<Arc<T>> for Arc<As>
where
    As: BitUnpackAsWithArgs<T> + ?Sized,
{
    type Args = As::Args;

    #[inline]
    fn unpack_as_with<R>(reader: R, args: Self::Args) -> Result<Arc<T>, R::Error>
    where
        R: BitReader,
    {
        AsWrap::<T, As>::unpack_with(reader, args)
            .map(AsWrap::into_inner)
            .map(Into::into)
    }
}

/// Implementation of [`Either X Y`](https://docs.ton.org/develop/data-formats/tl-b-types#either):
/// ```tlb
/// left$0 {X:Type} {Y:Type} value:X = Either X Y;
/// right$1 {X:Type} {Y:Type} value:Y = Either X Y;
/// ```
impl<Left, Right, AsLeft, AsRight> BitUnpackAsWithArgs<Either<Left, Right>>
    for Either<AsLeft, AsRight>
where
    AsLeft: BitUnpackAsWithArgs<Left>,
    AsRight: BitUnpackAsWithArgs<Right, Args = AsLeft::Args>,
{
    type Args = AsLeft::Args;

    #[inline]
    fn unpack_as_with<R>(reader: R, args: Self::Args) -> Result<Either<Left, Right>, R::Error>
    where
        R: BitReader,
    {
        Ok(
            Either::<AsWrap<Left, AsLeft>, AsWrap<Right, AsRight>>::unpack_with(reader, args)?
                .map_either(AsWrap::into_inner, AsWrap::into_inner),
        )
    }
}

impl<T, As> BitUnpackAsWithArgs<Option<T>> for Either<(), As>
where
    As: BitUnpackAsWithArgs<T>,
{
    type Args = As::Args;

    #[inline]
    fn unpack_as_with<R>(mut reader: R, args: Self::Args) -> Result<Option<T>, R::Error>
    where
        R: BitReader,
    {
        Ok(reader
            .unpack_as_with::<Either<(), T>, Either<NoArgs<_>, As>>(args)?
            .right())
    }
}

/// Implementation of [`Maybe X`](https://docs.ton.org/develop/data-formats/tl-b-types#maybe):
/// ```tlb
/// nothing$0 {X:Type} = Maybe X;
/// just$1 {X:Type} value:X = Maybe X;
/// ```
impl<T, As> BitUnpackAsWithArgs<Option<T>> for Option<As>
where
    As: BitUnpackAsWithArgs<T>,
{
    type Args = As::Args;

    #[inline]
    fn unpack_as_with<R>(reader: R, args: Self::Args) -> Result<Option<T>, R::Error>
    where
        R: BitReader,
    {
        Ok(Option::<AsWrap<T, As>>::unpack_with(reader, args)?.map(AsWrap::into_inner))
    }
}
