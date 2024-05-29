use core::marker::PhantomData;

use crate::{BitPackAs, BitPackAsWithArgs, BitReader, BitUnpackAs, BitUnpackAsWithArgs, Same};

pub struct NoArgs<Args, As: ?Sized = Same>(PhantomData<(Args, As)>);

impl<T, As, Args> BitPackAsWithArgs<T> for NoArgs<Args, As>
where
    As: BitPackAs<T> + ?Sized,
{
    type Args = ();

    fn pack_as_with<W>(source: &T, writer: W, _args: Self::Args) -> Result<(), W::Error>
    where
        W: crate::BitWriter,
    {
        As::pack_as(source, writer)
    }
}

impl<T, As, Args> BitUnpackAsWithArgs<T> for NoArgs<Args, As>
where
    As: BitUnpackAs<T> + ?Sized,
{
    type Args = ();

    fn unpack_as_with<R>(reader: R, _args: Self::Args) -> Result<T, R::Error>
    where
        R: BitReader,
    {
        As::unpack_as(reader)
    }
}

pub struct DefaultArgs<As: ?Sized = Same>(PhantomData<As>);

impl<T, As> BitPackAs<T> for DefaultArgs<As>
where
    As: BitPackAsWithArgs<T>,
    As::Args: Default,
{
    fn pack_as<W>(source: &T, writer: W) -> Result<(), W::Error>
    where
        W: crate::BitWriter,
    {
        As::pack_as_with(source, writer, <As::Args>::default())
    }
}

impl<T, As> BitUnpackAs<T> for DefaultArgs<As>
where
    As: BitUnpackAsWithArgs<T>,
    As::Args: Default,
{
    fn unpack_as<R>(reader: R) -> Result<T, R::Error>
    where
        R: BitReader,
    {
        As::unpack_as_with(reader, <As::Args>::default())
    }
}
