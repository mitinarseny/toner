use core::{fmt::Display, marker::PhantomData};

use crate::{
    de::{
        args::{r#as::BitUnpackAsWithArgs, BitUnpackWithArgs},
        r#as::BitUnpackAs,
        BitReader, BitUnpack,
    },
    ser::{
        args::{r#as::BitPackAsWithArgs, BitPackWithArgs},
        r#as::BitPackAs,
        BitPack, BitWriter,
    },
    Error,
};

/// Serialize value by converting it to/from a proxy type
/// with serialization support.
///
/// See [`TryFromInto`] for more generalized version of this adapter
/// which uses [`TryFrom`] trait instead
pub struct FromInto<T>(PhantomData<T>);

impl<T, As> BitPackAs<T> for FromInto<As>
where
    T: Into<As> + Clone,
    As: BitPack,
{
    #[inline]
    fn pack_as<W>(source: &T, writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
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
    fn pack_as_with<W>(source: &T, writer: W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        source.clone().into().pack_with(writer, args)
    }
}

impl<T, As> BitUnpackAs<T> for FromInto<As>
where
    As: Into<T> + BitUnpack,
{
    #[inline]
    fn unpack_as<R>(reader: R) -> Result<T, R::Error>
    where
        R: BitReader,
    {
        As::unpack(reader).map(Into::into)
    }
}

impl<T, As> BitUnpackAsWithArgs<T> for FromInto<As>
where
    As: Into<T> + BitUnpackWithArgs,
{
    type Args = As::Args;

    #[inline]
    fn unpack_as_with<R>(reader: R, args: Self::Args) -> Result<T, R::Error>
    where
        R: BitReader,
    {
        As::unpack_with(reader, args).map(Into::into)
    }
}

/// Serialize a reference value by converting it to/from a proxy type
/// with serialization support.
///
/// See [`TryFromIntoRef`] for more generalized version of this adapter
/// which uses [`TryFrom`] trait instead
pub struct FromIntoRef<T>(PhantomData<T>);

impl<T, As> BitPackAs<T> for FromIntoRef<As>
where
    for<'a> &'a T: Into<As>,
    As: BitPack,
{
    #[inline]
    fn pack_as<W>(source: &T, writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
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
    fn pack_as_with<W>(source: &T, writer: W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        source.into().pack_with(writer, args)
    }
}

impl<T, As> BitUnpackAs<T> for FromIntoRef<As>
where
    As: Into<T> + BitUnpack,
{
    #[inline]
    fn unpack_as<R>(reader: R) -> Result<T, R::Error>
    where
        R: BitReader,
    {
        As::unpack(reader).map(Into::into)
    }
}

impl<T, As> BitUnpackAsWithArgs<T> for FromIntoRef<As>
where
    As: Into<T> + BitUnpackWithArgs,
{
    type Args = As::Args;

    #[inline]
    fn unpack_as_with<R>(reader: R, args: Self::Args) -> Result<T, R::Error>
    where
        R: BitReader,
    {
        As::unpack_with(reader, args).map(Into::into)
    }
}

/// Serialize value by converting it to/from a proxy type
/// with serialization support.
///
/// **Note:** [`FromInto`] is more specialized version of this adapter
/// which the infailable [`Into`] trait instead.
pub struct TryFromInto<T>(PhantomData<T>);

impl<T, As> BitPackAs<T> for TryFromInto<As>
where
    T: TryInto<As> + Clone,
    <T as TryInto<As>>::Error: Display,
    As: BitPack,
{
    #[inline]
    fn pack_as<W>(source: &T, writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
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
    fn pack_as_with<W>(source: &T, writer: W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        source
            .clone()
            .try_into()
            .map_err(Error::custom)?
            .pack_with(writer, args)
    }
}

impl<T, As> BitUnpackAs<T> for TryFromInto<As>
where
    As: TryInto<T> + BitUnpack,
    <As as TryInto<T>>::Error: Display,
{
    #[inline]
    fn unpack_as<R>(reader: R) -> Result<T, R::Error>
    where
        R: BitReader,
    {
        As::unpack(reader)?.try_into().map_err(Error::custom)
    }
}

impl<T, As> BitUnpackAsWithArgs<T> for TryFromInto<As>
where
    As: TryInto<T> + BitUnpackWithArgs,
    <As as TryInto<T>>::Error: Display,
{
    type Args = As::Args;

    #[inline]
    fn unpack_as_with<R>(reader: R, args: Self::Args) -> Result<T, R::Error>
    where
        R: BitReader,
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
pub struct TryFromIntoRef<T>(PhantomData<T>);

impl<T, As> BitPackAs<T> for TryFromIntoRef<As>
where
    for<'a> &'a T: TryInto<As>,
    for<'a> <&'a T as TryInto<As>>::Error: Display,
    As: BitPack,
{
    #[inline]
    fn pack_as<W>(source: &T, writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
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
    fn pack_as_with<W>(source: &T, writer: W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        source
            .try_into()
            .map_err(Error::custom)?
            .pack_with(writer, args)
    }
}

impl<T, As> BitUnpackAs<T> for TryFromIntoRef<As>
where
    As: TryInto<T> + BitUnpack,
    <As as TryInto<T>>::Error: Display,
{
    #[inline]
    fn unpack_as<R>(reader: R) -> Result<T, R::Error>
    where
        R: BitReader,
    {
        As::unpack(reader)?.try_into().map_err(Error::custom)
    }
}

impl<T, As> BitUnpackAsWithArgs<T> for TryFromIntoRef<As>
where
    As: TryInto<T> + BitUnpackWithArgs,
    <As as TryInto<T>>::Error: Display,
{
    type Args = As::Args;

    #[inline]
    fn unpack_as_with<R>(reader: R, args: Self::Args) -> Result<T, R::Error>
    where
        R: BitReader,
    {
        As::unpack_with(reader, args)?
            .try_into()
            .map_err(Error::custom)
    }
}
