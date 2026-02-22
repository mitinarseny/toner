use core::{fmt::Display, marker::PhantomData};

use crate::{
    Error,
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
    #[inline]
    fn pack_as<W>(source: &T, writer: &mut W) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        source.clone().into().pack(writer)
    }
}

impl<T, As> BitPackAsWithArgs<T> for FromInto<As>
where
    T: Into<As> + Clone,
    As: BitPackWithArgs,
{
    type Args = As::Args;

    #[inline]
    fn pack_as_with<W>(source: &T, writer: &mut W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        source.clone().into().pack_with(writer, args)
    }
}

impl<'de, T, As> BitUnpackAs<'de, T> for FromInto<As>
where
    As: Into<T> + BitUnpack<'de>,
{
    #[inline]
    fn unpack_as<R>(reader: &mut R) -> Result<T, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        As::unpack(reader).map(Into::into)
    }
}

impl<'de, T, As> BitUnpackAsWithArgs<'de, T> for FromInto<As>
where
    As: Into<T> + BitUnpackWithArgs<'de>,
{
    type Args = As::Args;

    #[inline]
    fn unpack_as_with<R>(reader: &mut R, args: Self::Args) -> Result<T, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        As::unpack_with(reader, args).map(Into::into)
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
    #[inline]
    fn pack_as<W>(source: &T, writer: &mut W) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        source.into().pack(writer)
    }
}

impl<T, As> BitPackAsWithArgs<T> for FromIntoRef<As>
where
    for<'a> &'a T: Into<As>,
    As: BitPackWithArgs,
{
    type Args = As::Args;

    #[inline]
    fn pack_as_with<W>(source: &T, writer: &mut W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        source.into().pack_with(writer, args)
    }
}

impl<'de, T, As> BitUnpackAs<'de, T> for FromIntoRef<As>
where
    As: Into<T> + BitUnpack<'de>,
{
    #[inline]
    fn unpack_as<R>(reader: &mut R) -> Result<T, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        As::unpack(reader).map(Into::into)
    }
}

impl<'de, T, As> BitUnpackAsWithArgs<'de, T> for FromIntoRef<As>
where
    As: Into<T> + BitUnpackWithArgs<'de>,
{
    type Args = As::Args;

    #[inline]
    fn unpack_as_with<R>(reader: &mut R, args: Self::Args) -> Result<T, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        As::unpack_with(reader, args).map(Into::into)
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
    #[inline]
    fn pack_as<W>(source: &T, writer: &mut W) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        source
            .clone()
            .try_into()
            .map_err(Error::custom)?
            .pack(writer)
    }
}

impl<T, As> BitPackAsWithArgs<T> for TryFromInto<As>
where
    T: TryInto<As> + Clone,
    <T as TryInto<As>>::Error: Display,
    As: BitPackWithArgs,
{
    type Args = As::Args;

    #[inline]
    fn pack_as_with<W>(source: &T, writer: &mut W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        source
            .clone()
            .try_into()
            .map_err(Error::custom)?
            .pack_with(writer, args)
    }
}

impl<'de, T, As> BitUnpackAs<'de, T> for TryFromInto<As>
where
    As: TryInto<T> + BitUnpack<'de>,
    <As as TryInto<T>>::Error: Display,
{
    #[inline]
    fn unpack_as<R>(reader: &mut R) -> Result<T, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        As::unpack(reader)?.try_into().map_err(Error::custom)
    }
}

impl<'de, T, As> BitUnpackAsWithArgs<'de, T> for TryFromInto<As>
where
    As: TryInto<T> + BitUnpackWithArgs<'de>,
    <As as TryInto<T>>::Error: Display,
{
    type Args = As::Args;

    #[inline]
    fn unpack_as_with<R>(reader: &mut R, args: Self::Args) -> Result<T, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        As::unpack_with(reader, args)?
            .try_into()
            .map_err(Error::custom)
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
    #[inline]
    fn pack_as<W>(source: &T, writer: &mut W) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        source.try_into().map_err(Error::custom)?.pack(writer)
    }
}

impl<T, As> BitPackAsWithArgs<T> for TryFromIntoRef<As>
where
    for<'a> &'a T: TryInto<As>,
    for<'a> <&'a T as TryInto<As>>::Error: Display,
    As: BitPackWithArgs,
{
    type Args = As::Args;

    #[inline]
    fn pack_as_with<W>(source: &T, writer: &mut W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        source
            .try_into()
            .map_err(Error::custom)?
            .pack_with(writer, args)
    }
}

impl<'de, T, As> BitUnpackAs<'de, T> for TryFromIntoRef<As>
where
    As: TryInto<T> + BitUnpack<'de>,
    <As as TryInto<T>>::Error: Display,
{
    #[inline]
    fn unpack_as<R>(reader: &mut R) -> Result<T, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        As::unpack(reader)?.try_into().map_err(Error::custom)
    }
}

impl<'de, T, As> BitUnpackAsWithArgs<'de, T> for TryFromIntoRef<As>
where
    As: TryInto<T> + BitUnpackWithArgs<'de>,
    <As as TryInto<T>>::Error: Display,
{
    type Args = As::Args;

    #[inline]
    fn unpack_as_with<R>(reader: &mut R, args: Self::Args) -> Result<T, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        As::unpack_with(reader, args)?
            .try_into()
            .map_err(Error::custom)
    }
}
