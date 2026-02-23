//! **De**/**ser**ialization helpers for
//! [TL-B](https://docs.ton.org/develop/data-formats/tl-b-language).
//!
//! This approach is heavily inspired by
//! [serde_with](https://docs.rs/serde_with/latest/serde_with).
//! Please, read their docs for more usage examples.
mod args;
mod borrow;
mod default;
mod from_into;
mod integer;
mod len;
mod remainder;
mod same;
mod unary;

pub use self::{
    args::*, borrow::*, default::*, from_into::*, integer::*, len::*, remainder::*, same::*,
    unary::*,
};

use std::marker::PhantomData;

use impl_tools::autoimpl;

use crate::{
    de::{BitReader, BitUnpack, BitUnpackAs},
    ser::{BitPack, BitPackAs, BitWriter},
};

/// Helper to implement **de**/**ser**ialize trait for adapters
#[autoimpl(Clone where T: Clone)]
#[autoimpl(Copy where T: Copy)]
pub struct AsWrap<T, As>
where
    As: ?Sized,
{
    value: T,
    _phantom: PhantomData<As>,
}

impl<T, As> AsWrap<T, As>
where
    As: ?Sized,
{
    // Wrap given value
    #[inline]
    pub const fn new(value: T) -> Self {
        Self {
            value,
            _phantom: PhantomData,
        }
    }

    /// Unwrap inner value
    #[inline]
    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<T, As> BitPack for AsWrap<&T, As>
where
    T: ?Sized,
    As: BitPackAs<T> + ?Sized,
{
    type Args = As::Args;

    #[inline]
    fn pack<W>(&self, writer: &mut W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        As::pack_as(self.into_inner(), writer, args)
    }
}

impl<'de, T, As> BitUnpack<'de> for AsWrap<T, As>
where
    As: BitUnpackAs<'de, T> + ?Sized,
{
    type Args = As::Args;

    #[inline]
    fn unpack<R>(reader: &mut R, args: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        As::unpack_as(reader, args).map(Self::new)
    }
}
