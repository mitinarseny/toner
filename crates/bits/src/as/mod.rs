//! **De**/**ser**ialization helpers for
//! [TL-B](https://docs.ton.org/develop/data-formats/tl-b-language).
//!
//! This approach is heavily inspired by
//! [serde_with](https://docs.rs/serde_with/latest/serde_with).
//! Please, read their docs for more usage examples.
pub mod args;
mod bits;
mod borrow;
mod default;
mod from_into;
mod integer;
mod remainder;
mod same;
mod unary;

use std::marker::PhantomData;

use impl_tools::autoimpl;

use crate::{
    de::{
        BitReader, BitUnpack,
        args::{BitUnpackWithArgs, r#as::BitUnpackAsWithArgs},
        r#as::BitUnpackAs,
    },
    ser::{
        BitPack, BitWriter,
        args::{BitPackWithArgs, r#as::BitPackAsWithArgs},
        r#as::BitPackAs,
    },
};

pub use self::{
    bits::*, borrow::*, default::*, from_into::*, integer::*, remainder::*, same::*, unary::*,
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
    #[inline]
    fn pack<W>(&self, writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        As::pack_as(self.value, writer)
    }
}

impl<T, As> BitPackWithArgs for AsWrap<&T, As>
where
    T: ?Sized,
    As: BitPackAsWithArgs<T> + ?Sized,
{
    type Args = As::Args;

    #[inline]
    fn pack_with<W>(&self, writer: W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        As::pack_as_with(self.into_inner(), writer, args)
    }
}

impl<'de, T, As> BitUnpack<'de> for AsWrap<T, As>
where
    As: BitUnpackAs<'de, T> + ?Sized,
{
    #[inline]
    fn unpack<R>(reader: R) -> Result<Self, R::Error>
    where
        R: BitReader<'de>,
    {
        As::unpack_as(reader).map(|value| Self {
            value,
            _phantom: PhantomData,
        })
    }
}

impl<'de, T, As> BitUnpackWithArgs<'de> for AsWrap<T, As>
where
    As: BitUnpackAsWithArgs<'de, T> + ?Sized,
{
    type Args = As::Args;

    #[inline]
    fn unpack_with<R>(reader: R, args: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader<'de>,
    {
        As::unpack_as_with(reader, args).map(Self::new)
    }
}
