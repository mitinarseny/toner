pub mod args;
pub mod r#as;
mod reader;

pub use self::reader::*;

use core::mem::MaybeUninit;
use std::{rc::Rc, sync::Arc};

use bitvec::{order::Msb0, slice::BitSlice, view::AsBits};
use either::Either;

use crate::{
    r#as::{FromInto, Same},
    Error, ResultExt, StringError,
};

pub trait BitUnpack: Sized {
    fn unpack<R>(reader: R) -> Result<Self, R::Error>
    where
        R: BitReader;
}

#[inline]
pub fn unpack<T>(bits: impl AsRef<BitSlice<u8, Msb0>>) -> Result<T, StringError>
where
    T: BitUnpack,
{
    bits.as_ref().unpack()
}

#[inline]
pub fn unpack_bytes<T>(bytes: impl AsRef<[u8]>) -> Result<T, StringError>
where
    T: BitUnpack,
{
    unpack(bytes.as_bits())
}

#[inline]
pub fn unpack_fully<T>(bits: impl AsRef<BitSlice<u8, Msb0>>) -> Result<T, StringError>
where
    T: BitUnpack,
{
    let mut bits = bits.as_ref();
    let v = bits.unpack()?;
    if !bits.is_empty() {
        return Err(Error::custom("more data left"));
    }
    Ok(v)
}

#[inline]
pub fn unpack_bytes_fully<T>(bytes: impl AsRef<[u8]>) -> Result<T, StringError>
where
    T: BitUnpack,
{
    unpack_fully(bytes.as_bits())
}

impl BitUnpack for () {
    #[inline]
    fn unpack<R>(_reader: R) -> Result<Self, R::Error>
    where
        R: BitReader,
    {
        Ok(())
    }
}

impl BitUnpack for bool {
    #[inline]
    fn unpack<R>(mut reader: R) -> Result<Self, R::Error>
    where
        R: BitReader,
    {
        reader.read_bit()
    }
}

impl<T, const N: usize> BitUnpack for [T; N]
where
    T: BitUnpack,
{
    #[inline]
    fn unpack<R>(mut reader: R) -> Result<Self, R::Error>
    where
        R: BitReader,
    {
        let mut arr: [MaybeUninit<T>; N] = unsafe { MaybeUninit::uninit().assume_init() };
        for (i, a) in arr.iter_mut().enumerate() {
            a.write(T::unpack(&mut reader).with_context(|| format!("[{i}]"))?);
        }
        Ok(unsafe { arr.as_ptr().cast::<[T; N]>().read() })
    }
}

macro_rules! impl_bit_unpack_for_tuple {
    ($($n:tt:$t:ident),+) => {
        impl<$($t),+> BitUnpack for ($($t,)+)
        where $(
            $t: BitUnpack,
        )+
        {
            #[inline]
            fn unpack<R>(mut reader: R) -> Result<Self, R::Error>
            where
                R: BitReader,
            {
                Ok(($(
                    $t::unpack(&mut reader).context(concat!(".", stringify!($n)))?,
                )+))
            }
        }
    };
}
impl_bit_unpack_for_tuple!(0:T0);
impl_bit_unpack_for_tuple!(0:T0,1:T1);
impl_bit_unpack_for_tuple!(0:T0,1:T1,2:T2);
impl_bit_unpack_for_tuple!(0:T0,1:T1,2:T2,3:T3);
impl_bit_unpack_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4);
impl_bit_unpack_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5);
impl_bit_unpack_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6);
impl_bit_unpack_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6,7:T7);
impl_bit_unpack_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6,7:T7,8:T8);
impl_bit_unpack_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6,7:T7,8:T8,9:T9);

impl<T> BitUnpack for Box<T>
where
    T: BitUnpack,
{
    #[inline]
    fn unpack<R>(mut reader: R) -> Result<Self, R::Error>
    where
        R: BitReader,
    {
        reader.unpack_as::<_, FromInto<T>>()
    }
}

impl<T> BitUnpack for Rc<T>
where
    T: BitUnpack,
{
    #[inline]
    fn unpack<R>(mut reader: R) -> Result<Self, R::Error>
    where
        R: BitReader,
    {
        reader.unpack_as::<_, FromInto<T>>()
    }
}

impl<T> BitUnpack for Arc<T>
where
    T: BitUnpack,
{
    #[inline]
    fn unpack<R>(mut reader: R) -> Result<Self, R::Error>
    where
        R: BitReader,
    {
        reader.unpack_as::<_, FromInto<T>>()
    }
}

impl<Left, Right> BitUnpack for Either<Left, Right>
where
    Left: BitUnpack,
    Right: BitUnpack,
{
    #[inline]
    fn unpack<R>(mut reader: R) -> Result<Self, R::Error>
    where
        R: BitReader,
    {
        match reader.unpack().context("tag")? {
            false => reader.unpack().map(Either::Left).context("left"),
            true => reader.unpack().map(Either::Right).context("right"),
        }
    }
}

/// [Maybe](https://docs.ton.org/develop/data-formats/tl-b-types#maybe)
impl<T> BitUnpack for Option<T>
where
    T: BitUnpack,
{
    #[inline]
    fn unpack<R>(mut reader: R) -> Result<Self, R::Error>
    where
        R: BitReader,
    {
        reader.unpack_as::<_, Either<(), Same>>()
    }
}
