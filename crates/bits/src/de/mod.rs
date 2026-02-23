//! Binary **de**serialization for [TL-B](https://docs.ton.org/develop/data-formats/tl-b-language)
mod r#as;
mod reader;

pub use self::{r#as::*, reader::*};

use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet, LinkedList, VecDeque},
    hash::Hash,
    rc::Rc,
    sync::Arc,
};

use bitvec::{array::BitArray, order::Msb0, slice::BitSlice, vec::BitVec, view::BitViewSized};
use either::Either;

use crate::{
    Context, Error, StringError,
    r#as::{BorrowCow, FromInto, Same},
};

/// A type that can be bitwise-**de**serialized from any [`BitReader`].  
pub trait BitUnpack<'de>: Sized {
    /// Arguments to be passed in runtime
    type Args;

    /// Unpack the value with args
    fn unpack<R>(reader: &mut R, args: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader<'de> + ?Sized;
}

/// **De**serialize the value from [`BitSlice`]
#[inline]
pub fn unpack<'de, T>(mut bits: &'de BitSlice<u8, Msb0>, args: T::Args) -> Result<T, StringError>
where
    T: BitUnpack<'de>,
{
    bits.unpack(args)
}

/// **De**serialize the value from bytes slice
#[inline]
pub fn unpack_bytes<'de, T>(bytes: &'de [u8], args: T::Args) -> Result<T, StringError>
where
    T: BitUnpack<'de>,
{
    unpack(BitSlice::from_slice(bytes), args)
}

/// **De**serialize the value from [`BitSlice`] and ensure that no more data left.
#[inline]
pub fn unpack_fully<'de, T>(
    mut bits: &'de BitSlice<u8, Msb0>,
    args: T::Args,
) -> Result<T, StringError>
where
    T: BitUnpack<'de>,
{
    let v = bits.unpack(args)?;
    if !bits.is_empty() {
        return Err(Error::custom("more data left"));
    }
    Ok(v)
}

/// **De**serialize the value from bytes slice and ensure that no more data left.
#[inline]
pub fn unpack_bytes_fully<'de, T>(bytes: &'de [u8], args: T::Args) -> Result<T, StringError>
where
    T: BitUnpack<'de>,
{
    unpack_fully(BitSlice::from_slice(bytes), args)
}

impl<'de> BitUnpack<'de> for () {
    type Args = ();

    #[inline]
    fn unpack<R>(_reader: &mut R, _: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        Ok(())
    }
}

impl<'de> BitUnpack<'de> for bool {
    type Args = ();

    #[inline]
    fn unpack<R>(reader: &mut R, _: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        reader.read_bit()?.ok_or_else(|| Error::custom("EOF"))
    }
}

impl<'de, T, const N: usize> BitUnpack<'de> for [T; N]
where
    T: BitUnpack<'de>,
    T::Args: Clone,
{
    type Args = T::Args;

    #[inline]
    fn unpack<R>(reader: &mut R, args: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        // TODO: replace with [`core::array::try_from_fn`](https://github.com/rust-lang/rust/issues/89379) when stabilized
        array_util::try_from_fn(|i| {
            T::unpack(reader, args.clone()).with_context(|| format!("[{i}]"))
        })
    }
}

macro_rules! impl_bit_unpack_for_tuple {
    ($($n:tt:$t:ident),+) => {
        impl<'de, $($t),+> BitUnpack<'de> for ($($t,)+)
        where $(
            $t: BitUnpack<'de>,
        )+
        {
            type Args = ($($t::Args,)+);

            #[inline]
            fn unpack<R>(reader: &mut R, args: Self::Args) -> Result<Self, R::Error>
            where
                R: BitReader<'de> + ?Sized,
            {
                Ok(($(
                    $t::unpack(reader, args.$n).context(concat!(".", stringify!($n)))?,
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
    type Args = T::Args;

    #[inline]
    fn unpack<R>(reader: &mut R, args: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        reader.unpack_as::<_, FromInto<T>>(args)
    }
}

impl<'de, T> BitUnpack<'de> for Rc<T>
where
    T: BitUnpack<'de>,
{
    type Args = T::Args;

    #[inline]
    fn unpack<R>(reader: &mut R, args: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        reader.unpack_as::<_, FromInto<T>>(args)
    }
}

impl<'de, T> BitUnpack<'de> for Arc<T>
where
    T: BitUnpack<'de>,
{
    type Args = T::Args;

    #[inline]
    fn unpack<R>(reader: &mut R, args: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        reader.unpack_as::<_, FromInto<T>>(args)
    }
}

/// Always unpacks as [`Cow::Owned`]
impl<'de, T> BitUnpack<'de> for Cow<'_, T>
where
    T: ToOwned + ?Sized,
    T::Owned: BitUnpack<'de>,
{
    type Args = <T::Owned as BitUnpack<'de>>::Args;

    #[inline]
    fn unpack<R>(reader: &mut R, args: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        reader.unpack::<T::Owned>(args).map(Self::Owned)
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
    type Args = (Left::Args, Right::Args);

    #[inline]
    fn unpack<R>(reader: &mut R, (la, ra): Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        match reader.unpack(()).context("tag")? {
            false => reader.unpack(la).map(Either::Left).context("left"),
            true => reader.unpack(ra).map(Either::Right).context("right"),
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
    type Args = T::Args;

    #[inline]
    fn unpack<R>(reader: &mut R, args: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        reader.unpack_as::<_, Either<(), Same>>(args)
    }
}

impl<'de, A> BitUnpack<'de> for BitArray<A, Msb0>
where
    A: BitViewSized<Store = u8>,
{
    type Args = ();

    fn unpack<R>(reader: &mut R, _: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        let mut arr = Self::ZERO;
        reader.read_bits_into(arr.as_mut_bitslice())?;
        Ok(arr)
    }
}

impl<'de> BitUnpack<'de> for BitVec<u8, Msb0> {
    /// length in bits
    type Args = usize;

    #[inline]
    fn unpack<R>(reader: &mut R, len: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        reader
            .unpack_as::<Cow<BitSlice<u8, Msb0>>, BorrowCow>(len)
            .map(Cow::into_owned)
    }
}

impl<'de, T> BitUnpack<'de> for Vec<T>
where
    T: BitUnpack<'de>,
    T::Args: Clone,
{
    /// `(len, item_args)`
    type Args = (usize, T::Args);

    fn unpack<R>(reader: &mut R, (len, item_args): Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        reader.unpack_iter(item_args).take(len).collect()
    }
}

impl<'de, T> BitUnpack<'de> for VecDeque<T>
where
    T: BitUnpack<'de>,
    T::Args: Clone,
{
    /// `(len, item_args)`
    type Args = (usize, T::Args);

    fn unpack<R>(reader: &mut R, args: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        reader.unpack::<Vec<_>>(args).map(Into::into)
    }
}

impl<'de, T> BitUnpack<'de> for LinkedList<T>
where
    T: BitUnpack<'de>,
    T::Args: Clone,
{
    /// `(len, item_args)`
    type Args = (usize, T::Args);

    fn unpack<R>(reader: &mut R, (len, item_args): Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        reader.unpack_iter(item_args).take(len).collect()
    }
}

impl<'de, T> BitUnpack<'de> for BTreeSet<T>
where
    T: BitUnpack<'de> + Ord + Eq,
    T::Args: Clone,
{
    /// `(len, item_args)`
    type Args = (usize, T::Args);

    fn unpack<R>(reader: &mut R, (len, item_args): Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        reader.unpack_iter(item_args).take(len).collect()
    }
}

impl<'de, K, V> BitUnpack<'de> for BTreeMap<K, V>
where
    K: BitUnpack<'de> + Ord + Eq,
    K::Args: Clone,
    V: BitUnpack<'de>,
    V::Args: Clone,
{
    /// `(len, (key_args, value_args))`
    type Args = (usize, (K::Args, V::Args));

    fn unpack<R>(reader: &mut R, (len, kv): Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        reader.unpack_iter(kv).take(len).collect()
    }
}

impl<'de, T> BitUnpack<'de> for HashSet<T>
where
    T: BitUnpack<'de> + Hash + Eq,
    T::Args: Clone,
{
    /// `(len, item_args)`
    type Args = (usize, T::Args);

    fn unpack<R>(reader: &mut R, (len, item_args): Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        reader.unpack_iter(item_args).take(len).collect()
    }
}

impl<'de, K, V> BitUnpack<'de> for HashMap<K, V>
where
    K: BitUnpack<'de> + Hash + Eq,
    K::Args: Clone,
    V: BitUnpack<'de>,
    V::Args: Clone,
{
    /// `(len, (key_args, value_args))`
    type Args = (usize, (K::Args, V::Args));

    fn unpack<R>(reader: &mut R, (len, kv): Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        reader.unpack_iter(kv).take(len).collect()
    }
}
