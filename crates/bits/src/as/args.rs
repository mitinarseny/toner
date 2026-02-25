use core::{
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
    panic::{RefUnwindSafe, UnwindSafe},
};

use crate::{
    de::{BitReader, BitUnpackAs},
    ser::{BitPackAs, BitWriter},
};

use super::Same;

pub trait NoArgs:
    Sized
    + Send
    + Sync
    + Unpin
    + Debug
    + UnwindSafe
    + RefUnwindSafe
    + Clone
    + Copy
    + Default
    + PartialEq
    + Eq
    + Ord
    + PartialOrd
    + Hash
{
    const EMPTY: Self;
}

macro_rules! impl_no_args_for_tuple {
    (@impl $($t:ident),*) => {
        impl<$($t),*> NoArgs for ($($t,)*)
        where $(
            $t: NoArgs,
        )*{
            const EMPTY: Self = ($($t::EMPTY,)*);
        }
    };
    ($t:ident $(,$ts:ident)*) => {
        impl_no_args_for_tuple!($($ts),*);
        impl_no_args_for_tuple!(@impl $t $(,$ts)*);
    };
    () => {
        impl_no_args_for_tuple!(@impl);
    };
}
impl_no_args_for_tuple!(T9, T8, T7, T6, T5, T4, T3, T2, T1, T0);

/// Adapter to implement **de**/**ser**ialize with [`Default`] args.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DefaultArgs<As: ?Sized = Same>(PhantomData<As>);

impl<T, As> BitPackAs<T> for DefaultArgs<As>
where
    As: BitPackAs<T>,
    As::Args: Default,
{
    type Args = ();

    #[inline]
    fn pack_as<W>(source: &T, writer: &mut W, _: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        As::pack_as(source, writer, <As::Args>::default())
    }
}

impl<'de, T, As> BitUnpackAs<'de, T> for DefaultArgs<As>
where
    As: BitUnpackAs<'de, T>,
    As::Args: Default,
{
    type Args = ();

    #[inline]
    fn unpack_as<R>(reader: &mut R, _: Self::Args) -> Result<T, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        As::unpack_as(reader, <As::Args>::default())
    }
}
