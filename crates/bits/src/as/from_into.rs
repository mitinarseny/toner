use core::{fmt::Display, marker::PhantomData};

use crate::{BitPack, BitPackAs, BitReader, BitUnpack, BitUnpackAs, BitWriter, Error};

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
