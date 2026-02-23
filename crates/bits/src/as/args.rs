use core::marker::PhantomData;

use crate::{
    de::{BitReader, BitUnpackAs},
    ser::{BitPackAs, BitWriter},
};

use super::Same;

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
