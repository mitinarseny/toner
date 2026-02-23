use core::marker::PhantomData;

use crate::{
    de::{BitReader, BitReaderExt, BitUnpackAs},
    ser::{BitPackAs, BitWriter, BitWriterExt},
};

use super::Same;

/// **De**/**ser**ialize [`Default`] on `None` values
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DefaultOnNone<As: ?Sized = Same>(PhantomData<As>);

impl<T, As> BitPackAs<T> for DefaultOnNone<As>
where
    T: Default + PartialEq,
    As: BitPackAs<T>,
{
    type Args = As::Args;

    fn pack_as<W>(source: &T, writer: &mut W, args: As::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        writer.pack_as::<_, Option<&As>>((source != &T::default()).then_some(source), args)?;
        Ok(())
    }
}

impl<'de, T, As> BitUnpackAs<'de, T> for DefaultOnNone<As>
where
    T: Default,
    As: BitUnpackAs<'de, T>,
{
    type Args = As::Args;

    #[inline]
    fn unpack_as<R>(reader: &mut R, args: As::Args) -> Result<T, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        reader
            .unpack_as::<_, Option<As>>(args)
            .map(Option::unwrap_or_default)
    }
}
