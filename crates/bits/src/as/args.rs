use core::marker::PhantomData;

use crate::{
    de::{BitReader, args::r#as::BitUnpackAsWithArgs, r#as::BitUnpackAs},
    ser::{BitWriter, args::r#as::BitPackAsWithArgs, r#as::BitPackAs},
};

use super::Same;

/// Adapter to implement **de**/**ser**ialize with dynamic args for types
/// that do not require args for seralization.
pub struct NoArgs<Args, As: ?Sized = Same>(PhantomData<(Args, As)>);

impl<T, As, Args> BitPackAsWithArgs<T> for NoArgs<Args, As>
where
    As: BitPackAs<T> + ?Sized,
{
    type Args = Args;

    #[inline]
    fn pack_as_with<W>(source: &T, writer: W, _args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        As::pack_as(source, writer)
    }
}

impl<T, As, Args> BitUnpackAsWithArgs<T> for NoArgs<Args, As>
where
    As: BitUnpackAs<T> + ?Sized,
{
    type Args = Args;

    #[inline]
    fn unpack_as_with<R>(reader: R, _args: Self::Args) -> Result<T, R::Error>
    where
        R: BitReader,
    {
        As::unpack_as(reader)
    }
}

/// Adapter to implement **de**/**ser**ialize with [`Default`] args.
pub struct DefaultArgs<As: ?Sized = Same>(PhantomData<As>);

impl<T, As> BitPackAs<T> for DefaultArgs<As>
where
    As: BitPackAsWithArgs<T>,
    As::Args: Default,
{
    #[inline]
    fn pack_as<W>(source: &T, writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        As::pack_as_with(source, writer, <As::Args>::default())
    }
}

impl<T, As> BitUnpackAs<T> for DefaultArgs<As>
where
    As: BitUnpackAsWithArgs<T>,
    As::Args: Default,
{
    #[inline]
    fn unpack_as<R>(reader: R) -> Result<T, R::Error>
    where
        R: BitReader,
    {
        As::unpack_as_with(reader, <As::Args>::default())
    }
}
