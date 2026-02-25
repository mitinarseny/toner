use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet, LinkedList, VecDeque},
    hash::Hash,
    rc::Rc,
    sync::Arc,
};

use bitvec::{order::Msb0, slice::BitSlice};
use either::Either;

use crate::{
    Context, Error, StringError,
    r#as::{AsWrap, Same},
    de::BitUnpack,
};

use super::{BitReader, BitReaderExt};

/// Adapter to **de**serialize `T` with args.
///
/// This approach is heavily inspired by
/// [serde_with](https://docs.rs/serde_with/latest/serde_with).
/// Please, read their docs for more usage examples.
pub trait BitUnpackAs<'de, T> {
    type Args;

    /// Unpacks value with args using an adapter
    fn unpack_as<R>(reader: &mut R, args: Self::Args) -> Result<T, R::Error>
    where
        R: BitReader<'de> + ?Sized;
}
/// **De**serialize value from [`BitSlice`] with args using an adapter
#[inline]
pub fn unpack_as<'de, T, As>(
    mut bits: &'de BitSlice<u8, Msb0>,
    args: As::Args,
) -> Result<T, StringError>
where
    As: BitUnpackAs<'de, T>,
{
    bits.unpack_as::<_, As>(args)
}

/// **De**serialize value from bytes slice using an adapter
#[inline]
pub fn unpack_bytes_as<'de, T, As>(bytes: &'de [u8], args: As::Args) -> Result<T, StringError>
where
    As: BitUnpackAs<'de, T>,
{
    unpack_as::<_, As>(BitSlice::from_slice(bytes), args)
}

/// **De**serialize value from [`BitSlice`] using an adapter
/// and ensure that no more data left.
#[inline]
pub fn unpack_fully_as<'de, T, As>(
    mut bits: &'de BitSlice<u8, Msb0>,
    args: As::Args,
) -> Result<T, StringError>
where
    As: BitUnpackAs<'de, T>,
{
    let v = bits.unpack_as::<T, As>(args)?;
    if !bits.is_empty() {
        return Err(Error::custom("more data left"));
    }
    Ok(v)
}

/// **De**serialize value from bytes slice using an adapter
/// and ensure that no more data left.
#[inline]
pub fn unpack_bytes_fully_as<'de, T, As>(bytes: &'de [u8], args: As::Args) -> Result<T, StringError>
where
    As: BitUnpackAs<'de, T>,
{
    unpack_fully_as::<_, As>(BitSlice::from_slice(bytes), args)
}

impl<'de, T, As, const N: usize> BitUnpackAs<'de, [T; N]> for [As; N]
where
    As: BitUnpackAs<'de, T>,
    As::Args: Clone,
{
    type Args = As::Args;

    #[inline]
    fn unpack_as<R>(reader: &mut R, args: Self::Args) -> Result<[T; N], R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        // TODO: replace with [`core::array::try_from_fn`](https://github.com/rust-lang/rust/issues/89379) when stabilized
        array_util::try_from_fn(|i| {
            As::unpack_as(reader, args.clone()).with_context(|| format!("[{i}]"))
        })
    }
}

macro_rules! impl_bit_unpack_as_for_tuple {
    ($($ts:ident as $as:ident:$ns:tt),*) => {
        impl<'de, $($ts, $as),*> BitUnpackAs<'de, ($($ts,)*)> for ($($as,)*)
        where $(
            $as: BitUnpackAs<'de, $ts>,
        )*
        {
            type Args = ($($as::Args,)*);

            #[inline]
            #[allow(unused_variables)]
            fn unpack_as<R>(reader: &mut R, args: Self::Args) -> Result<($($ts,)*), R::Error>
            where
                R: BitReader<'de> + ?Sized,
            {
                Ok(($(
                    $as::unpack_as(reader, args.$ns)
                        .context(concat!(".", stringify!($ns)))?,
                )*))
            }
        }
    };
}
impl_bit_unpack_as_for_tuple!();
impl_bit_unpack_as_for_tuple!(T0 as As0:0);
impl_bit_unpack_as_for_tuple!(T0 as As0:0,T1 as As1:1);
impl_bit_unpack_as_for_tuple!(T0 as As0:0,T1 as As1:1,T2 as As2:2);
impl_bit_unpack_as_for_tuple!(T0 as As0:0,T1 as As1:1,T2 as As2:2,T3 as As3:3);
impl_bit_unpack_as_for_tuple!(T0 as As0:0,T1 as As1:1,T2 as As2:2,T3 as As3:3,T4 as As4:4);
impl_bit_unpack_as_for_tuple!(T0 as As0:0,T1 as As1:1,T2 as As2:2,T3 as As3:3,T4 as As4:4,T5 as As5:5);
impl_bit_unpack_as_for_tuple!(T0 as As0:0,T1 as As1:1,T2 as As2:2,T3 as As3:3,T4 as As4:4,T5 as As5:5,T6 as As6:6);
impl_bit_unpack_as_for_tuple!(T0 as As0:0,T1 as As1:1,T2 as As2:2,T3 as As3:3,T4 as As4:4,T5 as As5:5,T6 as As6:6,T7 as As7:7);
impl_bit_unpack_as_for_tuple!(T0 as As0:0,T1 as As1:1,T2 as As2:2,T3 as As3:3,T4 as As4:4,T5 as As5:5,T6 as As6:6,T7 as As7:7,T8 as As8:8);
impl_bit_unpack_as_for_tuple!(T0 as As0:0,T1 as As1:1,T2 as As2:2,T3 as As3:3,T4 as As4:4,T5 as As5:5,T6 as As6:6,T7 as As7:7,T8 as As8:8,T9 as As9:9);

impl<'de, T, As> BitUnpackAs<'de, Box<T>> for Box<As>
where
    As: BitUnpackAs<'de, T> + ?Sized,
{
    type Args = As::Args;

    #[inline]
    fn unpack_as<R>(reader: &mut R, args: Self::Args) -> Result<Box<T>, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        AsWrap::<T, As>::unpack(reader, args)
            .map(AsWrap::into_inner)
            .map(Into::into)
    }
}

impl<'de, T, As> BitUnpackAs<'de, Rc<T>> for Rc<As>
where
    As: BitUnpackAs<'de, T> + ?Sized,
{
    type Args = As::Args;

    #[inline]
    fn unpack_as<R>(reader: &mut R, args: Self::Args) -> Result<Rc<T>, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        AsWrap::<T, As>::unpack(reader, args)
            .map(AsWrap::into_inner)
            .map(Into::into)
    }
}

impl<'de, T, As> BitUnpackAs<'de, Arc<T>> for Arc<As>
where
    As: BitUnpackAs<'de, T> + ?Sized,
{
    type Args = As::Args;

    #[inline]
    fn unpack_as<R>(reader: &mut R, args: Self::Args) -> Result<Arc<T>, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        AsWrap::<T, As>::unpack(reader, args)
            .map(AsWrap::into_inner)
            .map(Into::into)
    }
}

/// Always unpacks as [`Cow::Owned`]
impl<'de, 'a, T, As> BitUnpackAs<'de, Cow<'a, T>> for Cow<'a, As>
where
    T: ToOwned + ?Sized,
    As: ToOwned + ?Sized,
    As::Owned: BitUnpackAs<'de, T::Owned>,
{
    type Args = <As::Owned as BitUnpackAs<'de, T::Owned>>::Args;

    #[inline]
    fn unpack_as<R>(reader: &mut R, args: Self::Args) -> Result<Cow<'a, T>, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        AsWrap::<T::Owned, As::Owned>::unpack(reader, args)
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
    type Args = (AsLeft::Args, AsRight::Args);

    #[inline]
    fn unpack_as<R>(reader: &mut R, args: Self::Args) -> Result<Either<Left, Right>, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        Ok(
            Either::<AsWrap<Left, AsLeft>, AsWrap<Right, AsRight>>::unpack(reader, args)?
                .map_either(AsWrap::into_inner, AsWrap::into_inner),
        )
    }
}

impl<'de, T, As> BitUnpackAs<'de, Option<T>> for Either<(), As>
where
    As: BitUnpackAs<'de, T>,
{
    type Args = As::Args;

    #[inline]
    fn unpack_as<R>(reader: &mut R, args: Self::Args) -> Result<Option<T>, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        Ok(reader
            .unpack_as::<Either<(), T>, Either<Same, As>>(((), args))?
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
    type Args = As::Args;

    #[inline]
    fn unpack_as<R>(reader: &mut R, args: Self::Args) -> Result<Option<T>, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        Ok(Option::<AsWrap<T, As>>::unpack(reader, args)?.map(AsWrap::into_inner))
    }
}

impl<'de, T, As> BitUnpackAs<'de, Vec<T>> for Vec<As>
where
    As: BitUnpackAs<'de, T>,
    As::Args: Clone,
{
    /// `(len, item_args)`
    type Args = (usize, As::Args);

    #[inline]
    fn unpack_as<R>(reader: &mut R, (len, args): Self::Args) -> Result<Vec<T>, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        reader.unpack_iter_as::<_, As>(args).take(len).collect()
    }
}

impl<'de, T, As> BitUnpackAs<'de, VecDeque<T>> for VecDeque<As>
where
    As: BitUnpackAs<'de, T>,
    As::Args: Clone,
{
    /// `(len, item_args)`
    type Args = (usize, As::Args);

    #[inline]
    fn unpack_as<R>(reader: &mut R, (len, args): Self::Args) -> Result<VecDeque<T>, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        reader.unpack_iter_as::<_, As>(args).take(len).collect()
    }
}

impl<'de, T, As> BitUnpackAs<'de, LinkedList<T>> for LinkedList<As>
where
    As: BitUnpackAs<'de, T>,
    As::Args: Clone,
{
    /// `(len, item_args)`
    type Args = (usize, As::Args);

    #[inline]
    fn unpack_as<R>(reader: &mut R, (len, args): Self::Args) -> Result<LinkedList<T>, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        reader.unpack_iter_as::<_, As>(args).take(len).collect()
    }
}

impl<'de, T, As> BitUnpackAs<'de, BTreeSet<T>> for BTreeSet<As>
where
    T: Ord + Eq,
    As: BitUnpackAs<'de, T>,
    As::Args: Clone,
{
    /// `(len, item_args)`
    type Args = (usize, As::Args);

    #[inline]
    fn unpack_as<R>(reader: &mut R, (len, args): Self::Args) -> Result<BTreeSet<T>, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        reader.unpack_iter_as::<_, As>(args).take(len).collect()
    }
}

impl<'de, K, V, KAs, VAs> BitUnpackAs<'de, BTreeMap<K, V>> for BTreeMap<KAs, VAs>
where
    K: Ord + Eq,
    KAs: BitUnpackAs<'de, K>,
    KAs::Args: Clone,
    VAs: BitUnpackAs<'de, V>,
    VAs::Args: Clone,
{
    /// `(len, (key_args, value_args))`
    type Args = (usize, (KAs::Args, VAs::Args));

    #[inline]
    fn unpack_as<R>(reader: &mut R, (len, args): Self::Args) -> Result<BTreeMap<K, V>, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        reader
            .unpack_iter_as::<_, (KAs, VAs)>(args)
            .take(len)
            .collect()
    }
}

impl<'de, T, As> BitUnpackAs<'de, HashSet<T>> for HashSet<As>
where
    T: Hash + Eq,
    As: BitUnpackAs<'de, T>,
    As::Args: Clone,
{
    /// `(len, item_args)`
    type Args = (usize, As::Args);

    #[inline]
    fn unpack_as<R>(reader: &mut R, (len, args): Self::Args) -> Result<HashSet<T>, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        reader.unpack_iter_as::<_, As>(args).take(len).collect()
    }
}

impl<'de, K, V, KAs, VAs> BitUnpackAs<'de, HashMap<K, V>> for HashMap<KAs, VAs>
where
    K: Hash + Eq,
    KAs: BitUnpackAs<'de, K>,
    KAs::Args: Clone,
    VAs: BitUnpackAs<'de, V>,
    VAs::Args: Clone,
{
    /// `(len, (key_args, value_args))`
    type Args = (usize, (KAs::Args, VAs::Args));

    #[inline]
    fn unpack_as<R>(reader: &mut R, (len, args): Self::Args) -> Result<HashMap<K, V>, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        reader
            .unpack_iter_as::<_, (KAs, VAs)>(args)
            .take(len)
            .collect()
    }
}
