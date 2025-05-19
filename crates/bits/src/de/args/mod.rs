pub mod r#as;

use core::mem::MaybeUninit;
use std::{rc::Rc, sync::Arc};

use bitvec::{mem::bits_of, order::Msb0, vec::BitVec};
use either::Either;

use crate::{
    Error, ResultExt,
    r#as::{FromInto, Same},
};

use super::{BitReader, BitReaderExt};

/// A type that can be bitwise-**de**serialized from any [`BitReader`].  
/// In contrast with [`BitUnpack`](super::BitUnpack) it allows to pass
/// [`Args`](BitUnpackWithArgs::Args) and these arguments can be
/// calculated dynamically in runtime.
pub trait BitUnpackWithArgs: Sized {
    type Args;

    /// Unpacks the value with args
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
    /// (len, T::Args)
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

/// Implementation of [`Either X Y`](https://docs.ton.org/develop/data-formats/tl-b-types#either):
/// ```tlb
/// left$0 {X:Type} {Y:Type} value:X = Either X Y;
/// right$1 {X:Type} {Y:Type} value:Y = Either X Y;
/// ```
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

/// Implementation of [`Maybe X`](https://docs.ton.org/develop/data-formats/tl-b-types#maybe):
/// ```tlb
/// nothing$0 {X:Type} = Maybe X;
/// just$1 {X:Type} value:X = Maybe X;
/// ```
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

impl BitUnpackWithArgs for BitVec<u8, Msb0> {
    /// length
    type Args = usize;

    #[inline]
    fn unpack_with<R>(mut reader: R, len: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader,
    {
        let mut dst = BitVec::with_capacity(len);
        dst.resize(len, false);
        let n = reader.read_bits_into(&mut dst)?;
        if n != len {
            return Err(Error::custom("EOF"));
        }
        Ok(dst)
    }
}

impl BitUnpackWithArgs for Vec<u8> {
    /// length
    type Args = usize;

    #[inline]
    fn unpack_with<R>(mut reader: R, len: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader,
    {
        let mut dst = vec![0; len];
        let n = reader.read_bytes_into(&mut dst)?;
        if n != len * bits_of::<u8>() {
            return Err(Error::custom("EOF"));
        }
        Ok(dst)
    }
}
