pub mod r#as;

use core::mem::MaybeUninit;
use std::{rc::Rc, sync::Arc};

use either::Either;

use crate::{
    r#as::{FromInto, Same},
    ResultExt,
};

use super::{BitReader, BitReaderExt};

pub trait BitUnpackWithArgs: Sized {
    type Args;

    fn unpack_with<R>(reader: R, args: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader;
}

impl<T, const N: usize> BitUnpackWithArgs for [T; N]
where
    T: BitUnpackWithArgs,
    T::Args: Clone,
{
    type Args = T::Args;

    #[inline]
    fn unpack_with<R>(mut reader: R, args: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader,
    {
        let mut arr: [MaybeUninit<T>; N] = unsafe { MaybeUninit::uninit().assume_init() };
        for (i, a) in arr.iter_mut().enumerate() {
            a.write(T::unpack_with(&mut reader, args.clone()).with_context(|| format!("[{i}]"))?);
        }
        Ok(unsafe { arr.as_ptr().cast::<[T; N]>().read() })
    }
}

macro_rules! impl_bit_unpack_with_args_for_tuple {
    ($($n:tt:$t:ident),+) => {
        impl<$($t),+> BitUnpackWithArgs for ($($t,)+)
        where $(
            $t: BitUnpackWithArgs,
        )+
        {
            type Args = ($($t::Args,)+);

            #[inline]
            fn unpack_with<R>(mut reader: R, args: Self::Args) -> Result<Self, R::Error>
            where
                R: BitReader,
            {
                Ok(($(
                    $t::unpack_with(&mut reader, args.$n).context(concat!(".", stringify!($n)))?,
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

impl<T> BitUnpackWithArgs for Vec<T>
where
    T: BitUnpackWithArgs,
    T::Args: Clone,
{
    type Args = (usize, T::Args);

    #[inline]
    fn unpack_with<R>(mut reader: R, (len, args): Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader,
    {
        reader.unpack_iter_with(args).take(len).collect()
    }
}

impl<T> BitUnpackWithArgs for Box<T>
where
    T: BitUnpackWithArgs,
{
    type Args = T::Args;

    #[inline]
    fn unpack_with<R>(mut reader: R, args: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader,
    {
        reader.unpack_as_with::<_, FromInto<T>>(args)
    }
}

impl<T> BitUnpackWithArgs for Rc<T>
where
    T: BitUnpackWithArgs,
{
    type Args = T::Args;

    #[inline]
    fn unpack_with<R>(mut reader: R, args: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader,
    {
        reader.unpack_as_with::<_, FromInto<T>>(args)
    }
}

impl<T> BitUnpackWithArgs for Arc<T>
where
    T: BitUnpackWithArgs,
{
    type Args = T::Args;

    #[inline]
    fn unpack_with<R>(mut reader: R, args: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader,
    {
        reader.unpack_as_with::<_, FromInto<T>>(args)
    }
}

impl<Left, Right> BitUnpackWithArgs for Either<Left, Right>
where
    Left: BitUnpackWithArgs,
    Right: BitUnpackWithArgs<Args = Left::Args>,
{
    type Args = Left::Args;

    #[inline]
    fn unpack_with<R>(mut reader: R, args: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader,
    {
        match reader.unpack().context("tag")? {
            false => reader.unpack_with(args).map(Either::Left).context("left"),
            true => reader.unpack_with(args).map(Either::Right).context("right"),
        }
    }
}

/// [Maybe](https://docs.ton.org/develop/data-formats/tl-b-types#maybe)
impl<T> BitUnpackWithArgs for Option<T>
where
    T: BitUnpackWithArgs,
{
    type Args = T::Args;

    #[inline]
    fn unpack_with<R>(mut reader: R, args: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader,
    {
        reader.unpack_as_with::<_, Either<(), Same>>(args)
    }
}
