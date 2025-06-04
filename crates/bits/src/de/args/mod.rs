pub mod r#as;

use core::mem::MaybeUninit;
use std::{borrow::Cow, rc::Rc, sync::Arc};

use bitvec::{mem::bits_of, order::Msb0, vec::BitVec};
use either::Either;

use crate::{
    Context, Error,
    r#as::{FromInto, Same},
};

use super::{BitReader, BitReaderExt};

/// A type that can be bitwise-**de**serialized from any [`BitReader`].  
/// In contrast with [`BitUnpack`](super::BitUnpack) it allows to pass
/// [`Args`](BitUnpackWithArgs::Args) and these arguments can be
/// calculated dynamically in runtime.
pub trait BitUnpackWithArgs<'de>: Sized {
    type Args;

    /// Unpacks the value with args
    fn unpack_with<R>(reader: R, args: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader<'de>;
}

impl<'de, T, const N: usize> BitUnpackWithArgs<'de> for [T; N]
where
    T: BitUnpackWithArgs<'de>,
    T::Args: Clone,
{
    type Args = T::Args;

    #[inline]
    fn unpack_with<R>(mut reader: R, args: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader<'de>,
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
        impl<'de, $($t),+> BitUnpackWithArgs<'de> for ($($t,)+)
        where $(
            $t: BitUnpackWithArgs<'de>,
        )+
        {
            type Args = ($($t::Args,)+);

            #[inline]
            fn unpack_with<R>(mut reader: R, args: Self::Args) -> Result<Self, R::Error>
            where
                R: BitReader<'de>,
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

impl<'de, T> BitUnpackWithArgs<'de> for Vec<T>
where
    T: BitUnpackWithArgs<'de>,
    T::Args: Clone,
{
    /// (len, T::Args)
    type Args = (usize, T::Args);

    #[inline]
    fn unpack_with<R>(mut reader: R, (len, args): Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader<'de>,
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
    fn unpack_with<R>(mut reader: R, args: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader<'de>,
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
    fn unpack_with<R>(mut reader: R, args: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader<'de>,
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
    fn unpack_with<R>(mut reader: R, args: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader<'de>,
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
    fn unpack_with<R>(mut reader: R, args: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader<'de>,
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
    fn unpack_with<R>(mut reader: R, args: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader<'de>,
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
    fn unpack_with<R>(mut reader: R, args: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader<'de>,
    {
        reader.unpack_as_with::<_, Either<(), Same>>(args)
    }
}

impl<'de> BitUnpackWithArgs<'de> for BitVec<u8, Msb0> {
    /// length
    type Args = usize;

    #[inline]
    fn unpack_with<R>(mut reader: R, len: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader<'de>,
    {
        // let v = reader.unpack_as_with::<>(args)
        let v = reader.read_bits(len)?;
        if v.len() != len {
            return Err(Error::custom("EOF"));
        }
        Ok(v.into_owned())
    }
}

impl<'de> BitUnpackWithArgs<'de> for Vec<u8> {
    /// length
    type Args = usize;

    #[inline]
    fn unpack_with<R>(mut reader: R, len: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader<'de>,
    {
        let mut dst = vec![0; len];
        let n = reader.read_bytes_into(&mut dst)?;
        if n != len * bits_of::<u8>() {
            return Err(Error::custom("EOF"));
        }
        Ok(dst)
    }
}
