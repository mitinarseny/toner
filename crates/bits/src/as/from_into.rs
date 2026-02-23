use core::{fmt::Display, marker::PhantomData};

use crate::{
    Error,
    de::{BitReader, BitUnpack, BitUnpackAs},
    ser::{BitPack, BitPackAs, BitWriter},
};

/// Serialize value by converting it to/from a proxy type
/// with serialization support.
///
/// See [`TryFromInto`] for more generalized version of this adapter
/// which uses [`TryFrom`] trait instead
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FromInto<T>(PhantomData<T>);

impl<T, As> BitPackAs<T> for FromInto<As>
where
    T: Into<As> + Clone,
    As: BitPack,
{
    type Args = As::Args;

    #[inline]
    fn pack_as<W>(source: &T, writer: &mut W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        source.clone().into().pack(writer, args)
    }
}

impl<'de, T, As> BitUnpackAs<'de, T> for FromInto<As>
where
    As: Into<T> + BitUnpack<'de>,
{
    type Args = As::Args;

    #[inline]
    fn unpack_as<R>(reader: &mut R, args: Self::Args) -> Result<T, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        As::unpack(reader, args).map(Into::into)
    }
}

/// Serialize a reference value by converting it to/from a proxy type
/// with serialization support.
///
/// See [`TryFromIntoRef`] for more generalized version of this adapter
/// which uses [`TryFrom`] trait instead
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FromIntoRef<T>(PhantomData<T>);

impl<T, As> BitPackAs<T> for FromIntoRef<As>
where
    for<'a> &'a T: Into<As>,
    As: BitPack,
{
    type Args = As::Args;

    #[inline]
    fn pack_as<W>(source: &T, writer: &mut W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        source.into().pack(writer, args)
    }
}

impl<'de, T, As> BitUnpackAs<'de, T> for FromIntoRef<As>
where
    As: Into<T> + BitUnpack<'de>,
{
    type Args = As::Args;

    #[inline]
    fn unpack_as<R>(reader: &mut R, args: Self::Args) -> Result<T, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        As::unpack(reader, args).map(Into::into)
    }
}

/// Serialize value by converting it to/from a proxy type
/// with serialization support.
///
/// **Note:** [`FromInto`] is more specialized version of this adapter
/// which the infailable [`Into`] trait instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TryFromInto<T>(PhantomData<T>);

impl<T, As> BitPackAs<T> for TryFromInto<As>
where
    T: TryInto<As> + Clone,
    <T as TryInto<As>>::Error: Display,
    As: BitPack,
{
    type Args = As::Args;

    #[inline]
    fn pack_as<W>(source: &T, writer: &mut W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        source
            .clone()
            .try_into()
            .map_err(Error::custom)?
            .pack(writer, args)
    }
}

impl<'de, T, As> BitUnpackAs<'de, T> for TryFromInto<As>
where
    As: TryInto<T> + BitUnpack<'de>,
    <As as TryInto<T>>::Error: Display,
{
    type Args = As::Args;

    #[inline]
    fn unpack_as<R>(reader: &mut R, args: Self::Args) -> Result<T, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        As::unpack(reader, args)?.try_into().map_err(Error::custom)
    }
}

/// Serialize a reference value by converting it to/from a proxy type
/// with serialization support.
///
/// **Note:** [`FromIntoRef`] is more specialized version of this adapter
/// which the infailable [`Into`] trait instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TryFromIntoRef<T>(PhantomData<T>);

impl<T, As> BitPackAs<T> for TryFromIntoRef<As>
where
    for<'a> &'a T: TryInto<As>,
    for<'a> <&'a T as TryInto<As>>::Error: Display,
    As: BitPack,
{
    type Args = As::Args;

    #[inline]
    fn pack_as<W>(source: &T, writer: &mut W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        source.try_into().map_err(Error::custom)?.pack(writer, args)
    }
}

impl<'de, T, As> BitUnpackAs<'de, T> for TryFromIntoRef<As>
where
    As: TryInto<T> + BitUnpack<'de>,
    <As as TryInto<T>>::Error: Display,
{
    type Args = As::Args;

    #[inline]
    fn unpack_as<R>(reader: &mut R, args: Self::Args) -> Result<T, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        As::unpack(reader, args)?.try_into().map_err(Error::custom)
    }
}
