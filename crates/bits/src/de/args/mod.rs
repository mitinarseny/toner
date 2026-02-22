pub mod r#as;

use std::{borrow::Cow, rc::Rc, sync::Arc};

use bitvec::{order::Msb0, slice::BitSlice, vec::BitVec};
use either::Either;

use crate::{
    Context,
    r#as::{BorrowCow, FromInto, Same},
};

use super::{BitReader, BitReaderExt};

/// A type that can be bitwise-**de**serialized from any [`BitReader`].  
/// In contrast with [`BitUnpack`](super::BitUnpack) it allows to pass
/// [`Args`](BitUnpackWithArgs::Args) and these arguments can be
/// calculated dynamically in runtime.
pub trait BitUnpackWithArgs<'de>: Sized {
    type Args;

    /// Unpacks the value with args
    fn unpack_with<R>(reader: &mut R, args: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader<'de> + ?Sized;
}

impl<'de, T, const N: usize> BitUnpackWithArgs<'de> for [T; N]
where
    T: BitUnpackWithArgs<'de>,
    T::Args: Clone,
{
    type Args = T::Args;

    #[inline]
    fn unpack_with<R>(reader: &mut R, args: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        // TODO: replace with [`core::array::try_from_fn`](https://github.com/rust-lang/rust/issues/89379) when stabilized
        array_util::try_from_fn(|i| {
            T::unpack_with(reader, args.clone()).with_context(|| format!("[{i}]"))
        })
    }
}

macro_rules! impl_bit_unpack_with_args_for_tuple {
    ($($n:tt:$t:ident),+) => {
        impl<'de, $($t),+> BitUnpackWithArgs<'de> for ($($t,)+)
        where $(
            $t: BitUnpackWithArgs<'de>,
        )+
        {
            type Args = ($($t::Args,)+);

            #[inline]
            fn unpack_with<R>(reader: &mut R, args: Self::Args) -> Result<Self, R::Error>
            where
                R: BitReader<'de> + ?Sized,
            {
                Ok(($(
                    $t::unpack_with(reader, args.$n).context(concat!(".", stringify!($n)))?,
                )+))
            }
        }
    };
}
impl_bit_unpack_with_args_for_tuple!(0:T0);
impl_bit_unpack_with_args_for_tuple!(0:T0,1:T1);
impl_bit_unpack_with_args_for_tuple!(0:T0,1:T1,2:T2);
impl_bit_unpack_with_args_for_tuple!(0:T0,1:T1,2:T2,3:T3);
impl_bit_unpack_with_args_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4);
impl_bit_unpack_with_args_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5);
impl_bit_unpack_with_args_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6);
impl_bit_unpack_with_args_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6,7:T7);
impl_bit_unpack_with_args_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6,7:T7,8:T8);
impl_bit_unpack_with_args_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6,7:T7,8:T8,9:T9);

impl<'de, T> BitUnpackWithArgs<'de> for Vec<T>
where
    T: BitUnpackWithArgs<'de>,
    T::Args: Clone,
{
    /// (len, T::Args)
    type Args = (usize, T::Args);

    #[inline]
    fn unpack_with<R>(reader: &mut R, (len, args): Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        reader.unpack_iter_with(args).take(len).collect()
    }
}

impl<'de, T> BitUnpackWithArgs<'de> for Box<T>
where
    T: BitUnpackWithArgs<'de>,
{
    type Args = T::Args;

    #[inline]
    fn unpack_with<R>(reader: &mut R, args: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        reader.unpack_as_with::<_, FromInto<T>>(args)
    }
}

impl<'de, T> BitUnpackWithArgs<'de> for Rc<T>
where
    T: BitUnpackWithArgs<'de>,
{
    type Args = T::Args;

    #[inline]
    fn unpack_with<R>(reader: &mut R, args: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        reader.unpack_as_with::<_, FromInto<T>>(args)
    }
}

impl<'de, T> BitUnpackWithArgs<'de> for Arc<T>
where
    T: BitUnpackWithArgs<'de>,
{
    type Args = T::Args;

    #[inline]
    fn unpack_with<R>(reader: &mut R, args: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        reader.unpack_as_with::<_, FromInto<T>>(args)
    }
}

/// Always unpacks as [`Cow::Owned`]
impl<'de, T> BitUnpackWithArgs<'de> for Cow<'_, T>
where
    T: ToOwned + ?Sized,
    T::Owned: BitUnpackWithArgs<'de>,
{
    type Args = <T::Owned as BitUnpackWithArgs<'de>>::Args;

    #[inline]
    fn unpack_with<R>(reader: &mut R, args: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        reader.unpack_with::<T::Owned>(args).map(Self::Owned)
    }
}

/// Implementation of [`Either X Y`](https://docs.ton.org/develop/data-formats/tl-b-types#either):
/// ```tlb
/// left$0 {X:Type} {Y:Type} value:X = Either X Y;
/// right$1 {X:Type} {Y:Type} value:Y = Either X Y;
/// ```
impl<'de, Left, Right> BitUnpackWithArgs<'de> for Either<Left, Right>
where
    Left: BitUnpackWithArgs<'de>,
    Right: BitUnpackWithArgs<'de, Args = Left::Args>,
{
    type Args = Left::Args;

    #[inline]
    fn unpack_with<R>(reader: &mut R, args: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        match reader.unpack().context("tag")? {
            false => reader.unpack_with(args).map(Either::Left).context("left"),
            true => reader.unpack_with(args).map(Either::Right).context("right"),
        }
    }
}

/// Implementation of [`Maybe X`](https://docs.ton.org/develop/data-formats/tl-b-types#maybe):
/// ```tlb
/// nothing$0 {X:Type} = Maybe X;
/// just$1 {X:Type} value:X = Maybe X;
/// ```
impl<'de, T> BitUnpackWithArgs<'de> for Option<T>
where
    T: BitUnpackWithArgs<'de>,
{
    type Args = T::Args;

    #[inline]
    fn unpack_with<R>(reader: &mut R, args: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        reader.unpack_as_with::<_, Either<(), Same>>(args)
    }
}

impl<'de> BitUnpackWithArgs<'de> for BitVec<u8, Msb0> {
    /// length in bits
    type Args = usize;

    #[inline]
    fn unpack_with<R>(reader: &mut R, len: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        reader
            .unpack_as_with::<Cow<BitSlice<u8, Msb0>>, BorrowCow>(len)
            .map(Cow::into_owned)
    }
}

impl<'de> BitUnpackWithArgs<'de> for Vec<u8> {
    /// length in bytes
    type Args = usize;

    #[inline]
    fn unpack_with<R>(reader: &mut R, len: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        reader
            .unpack_as_with::<Cow<[u8]>, BorrowCow>(len)
            .map(Cow::into_owned)
    }
}
