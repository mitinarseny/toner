use core::{marker::PhantomData, mem::MaybeUninit};
use std::{rc::Rc, sync::Arc};

use bitvec::{order::Msb0, slice::BitSlice, view::AsBits};

use crate::{BitReader, BitReaderExt, BitUnpack, Error, ResultExt, StringError};

pub trait BitUnpackAs<T> {
    fn unpack_as<R>(reader: R) -> Result<T, R::Error>
    where
        R: BitReader;
}

#[inline]
pub fn unpack_as<T, As>(bits: impl AsRef<BitSlice<u8, Msb0>>) -> Result<T, StringError>
where
    As: BitUnpackAs<T>,
{
    bits.as_ref().unpack_as::<T, As>()
}

#[inline]
pub fn unpack_bytes_as<T, As>(bytes: impl AsRef<[u8]>) -> Result<T, StringError>
where
    As: BitUnpackAs<T>,
{
    unpack_as::<_, As>(bytes.as_bits())
}

#[inline]
pub fn unpack_fully_as<T, As>(bits: impl AsRef<BitSlice<u8, Msb0>>) -> Result<T, StringError>
where
    As: BitUnpackAs<T>,
{
    let mut bits = bits.as_ref();
    let v = bits.unpack_as::<T, As>()?;
    if !bits.is_empty() {
        return Err(Error::custom("more data left"));
    }
    Ok(v)
}

#[inline]
pub fn unpack_bytes_fully_as<T, As>(bytes: impl AsRef<[u8]>) -> Result<T, StringError>
where
    As: BitUnpackAs<T>,
{
    unpack_fully_as::<_, As>(bytes.as_bits())
}

pub struct BitUnpackAsWrap<T, As>
where
    As: ?Sized,
{
    value: T,
    _phanton: PhantomData<As>,
}

impl<T, As> BitUnpackAsWrap<T, As>
where
    As: BitUnpackAs<T> + ?Sized,
{
    /// Return the inner value of type `T`.
    #[inline]
    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<T, As> BitUnpack for BitUnpackAsWrap<T, As>
where
    As: BitUnpackAs<T> + ?Sized,
{
    #[inline]
    fn unpack<R>(reader: R) -> Result<Self, R::Error>
    where
        R: BitReader,
    {
        As::unpack_as(reader).map(|value| Self {
            value,
            _phanton: PhantomData,
        })
    }
}

impl<T, As, const N: usize> BitUnpackAs<[T; N]> for [As; N]
where
    As: BitUnpackAs<T>,
{
    #[inline]
    fn unpack_as<R>(mut reader: R) -> Result<[T; N], R::Error>
    where
        R: BitReader,
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
        impl<$($t, $a),+> BitUnpackAs<($($t,)+)> for ($($a,)+)
        where $(
            $a: BitUnpackAs<$t>,
        )+
        {
            #[inline]
            fn unpack_as<R>(mut reader: R) -> Result<($($t,)+), R::Error>
            where
                R: BitReader,
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

impl<T, As> BitUnpackAs<Box<T>> for Box<As>
where
    As: BitUnpackAs<T> + ?Sized,
{
    #[inline]
    fn unpack_as<R>(reader: R) -> Result<Box<T>, R::Error>
    where
        R: BitReader,
    {
        BitUnpackAsWrap::<T, As>::unpack(reader)
            .map(BitUnpackAsWrap::into_inner)
            .map(Box::new)
    }
}

impl<T, As> BitUnpackAs<Rc<T>> for Rc<As>
where
    As: BitUnpackAs<T> + ?Sized,
{
    #[inline]
    fn unpack_as<R>(reader: R) -> Result<Rc<T>, R::Error>
    where
        R: BitReader,
    {
        BitUnpackAsWrap::<T, As>::unpack(reader)
            .map(BitUnpackAsWrap::into_inner)
            .map(Rc::new)
    }
}

impl<T, As> BitUnpackAs<Arc<T>> for Arc<As>
where
    As: BitUnpackAs<T> + ?Sized,
{
    #[inline]
    fn unpack_as<R>(reader: R) -> Result<Arc<T>, R::Error>
    where
        R: BitReader,
    {
        BitUnpackAsWrap::<T, As>::unpack(reader)
            .map(BitUnpackAsWrap::into_inner)
            .map(Arc::new)
    }
}

impl<T, As> BitUnpackAs<Option<T>> for Option<As>
where
    As: BitUnpackAs<T>,
{
    #[inline]
    fn unpack_as<R>(reader: R) -> Result<Option<T>, R::Error>
    where
        R: BitReader,
    {
        Ok(Option::<BitUnpackAsWrap<T, As>>::unpack(reader)?.map(BitUnpackAsWrap::into_inner))
    }
}
