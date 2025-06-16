//! Binary **de**serialization for [TL-B](https://docs.ton.org/develop/data-formats/tl-b-language)
pub mod args;
pub mod r#as;
mod reader;

pub use self::reader::*;

use std::{borrow::Cow, rc::Rc, sync::Arc};

use bitvec::{order::Msb0, slice::BitSlice};
use either::Either;

use crate::{
    Context, Error, StringError,
    r#as::{FromInto, Same},
};

/// A type that can be bitwise-**de**serialized from any [`BitReader`].
pub trait BitUnpack<'de>: Sized {
    /// Unpack value from the reader.
    fn unpack<R>(reader: R) -> Result<Self, R::Error>
    where
        R: BitReader<'de>;
}

/// **De**serialize the value from [`BitSlice`]
#[inline]
pub fn unpack<'de, T>(mut bits: &'de BitSlice<u8, Msb0>) -> Result<T, StringError>
where
    T: BitUnpack<'de>,
{
    bits.unpack()
}

/// **De**serialize the value from bytes slice
#[inline]
pub fn unpack_bytes<'de, T>(bytes: &'de [u8]) -> Result<T, StringError>
where
    T: BitUnpack<'de>,
{
    unpack(BitSlice::from_slice(bytes))
}

/// **De**serialize the value from [`BitSlice`] and ensure that no more data left.
#[inline]
pub fn unpack_fully<'de, T>(mut bits: &'de BitSlice<u8, Msb0>) -> Result<T, StringError>
where
    T: BitUnpack<'de>,
{
    let v = bits.unpack()?;
    if !bits.is_empty() {
        return Err(Error::custom("more data left"));
    }
    Ok(v)
}

/// **De**serialize the value from bytes slice and ensure that no more data left.
#[inline]
pub fn unpack_bytes_fully<'de, T>(bytes: &'de [u8]) -> Result<T, StringError>
where
    T: BitUnpack<'de>,
{
    unpack_fully(BitSlice::from_slice(bytes))
}

impl<'de> BitUnpack<'de> for () {
    #[inline]
    fn unpack<R>(_reader: R) -> Result<Self, R::Error>
    where
        R: BitReader<'de>,
    {
        Ok(())
    }
}

impl<'de> BitUnpack<'de> for bool {
    #[inline]
    fn unpack<R>(mut reader: R) -> Result<Self, R::Error>
    where
        R: BitReader<'de>,
    {
        reader.read_bit()?.ok_or_else(|| Error::custom("EOF"))
    }
}

impl<'de, T, const N: usize> BitUnpack<'de> for [T; N]
where
    T: BitUnpack<'de>,
{
    #[inline]
    fn unpack<R>(mut reader: R) -> Result<Self, R::Error>
    where
        R: BitReader<'de>,
    {
        // TODO: replace with [`core::array::try_from_fn`](https://github.com/rust-lang/rust/issues/89379) when stabilized
        array_util::try_from_fn(|i| T::unpack(&mut reader).with_context(|| format!("[{i}]")))
    }
}

macro_rules! impl_bit_unpack_for_tuple {
    ($($n:tt:$t:ident),+) => {
        impl<'de, $($t),+> BitUnpack<'de> for ($($t,)+)
        where $(
            $t: BitUnpack<'de>,
        )+
        {
            #[inline]
            fn unpack<R>(mut reader: R) -> Result<Self, R::Error>
            where
                R: BitReader<'de>,
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

impl<'de, T> BitUnpack<'de> for Box<T>
where
    T: BitUnpack<'de>,
{
    #[inline]
    fn unpack<R>(mut reader: R) -> Result<Self, R::Error>
    where
        R: BitReader<'de>,
    {
        reader.unpack_as::<_, FromInto<T>>()
    }
}

impl<'de, T> BitUnpack<'de> for Rc<T>
where
    T: BitUnpack<'de>,
{
    #[inline]
    fn unpack<R>(mut reader: R) -> Result<Self, R::Error>
    where
        R: BitReader<'de>,
    {
        reader.unpack_as::<_, FromInto<T>>()
    }
}

impl<'de, T> BitUnpack<'de> for Arc<T>
where
    T: BitUnpack<'de>,
{
    #[inline]
    fn unpack<R>(mut reader: R) -> Result<Self, R::Error>
    where
        R: BitReader<'de>,
    {
        reader.unpack_as::<_, FromInto<T>>()
    }
}

/// Always unpacks as [`Cow::Owned`]
impl<'de, T> BitUnpack<'de> for Cow<'_, T>
where
    T: ToOwned + ?Sized,
    T::Owned: BitUnpack<'de>,
{
    #[inline]
    fn unpack<R>(reader: R) -> Result<Self, R::Error>
    where
        R: BitReader<'de>,
    {
        <T::Owned as BitUnpack>::unpack(reader).map(Self::Owned)
    }
}

/// Implementation of [`Either X Y`](https://docs.ton.org/develop/data-formats/tl-b-types#either):
/// ```tlb
/// left$0 {X:Type} {Y:Type} value:X = Either X Y;
/// right$1 {X:Type} {Y:Type} value:Y = Either X Y;
/// ```
impl<'de, Left, Right> BitUnpack<'de> for Either<Left, Right>
where
    Left: BitUnpack<'de>,
    Right: BitUnpack<'de>,
{
    #[inline]
    fn unpack<R>(mut reader: R) -> Result<Self, R::Error>
    where
        R: BitReader<'de>,
    {
        match reader.unpack().context("tag")? {
            false => reader.unpack().map(Either::Left).context("left"),
            true => reader.unpack().map(Either::Right).context("right"),
        }
    }
}

/// Implementation of [`Maybe X`](https://docs.ton.org/develop/data-formats/tl-b-types#maybe):
/// ```tlb
/// nothing$0 {X:Type} = Maybe X;
/// just$1 {X:Type} value:X = Maybe X;
/// ```
impl<'de, T> BitUnpack<'de> for Option<T>
where
    T: BitUnpack<'de>,
{
    #[inline]
    fn unpack<R>(mut reader: R) -> Result<Self, R::Error>
    where
        R: BitReader<'de>,
    {
        reader.unpack_as::<_, Either<(), Same>>()
    }
}
