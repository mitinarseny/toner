use core::marker::PhantomData;

use crate::{
    de::{BitReader, BitReaderExt, r#as::BitUnpackAs},
    ser::{BitWriter, BitWriterExt, r#as::BitPackAs},
};

use super::Same;

/// **De**/**ser**ialize [`Default`] on `None` values
pub struct DefaultOnNone<As: ?Sized = Same>(PhantomData<As>);

impl<T, As> BitPackAs<Option<T>> for DefaultOnNone<As>
where
    T: Default,
    As: BitPackAs<T>,
{
    fn pack_as<W>(source: &Option<T>, mut writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        match source {
            Some(v) => writer.pack_as::<_, &As>(v)?,
            None => writer.pack_as::<_, As>(T::default())?,
        };
        Ok(())
    }
}

impl<T, As> BitUnpackAs<T> for DefaultOnNone<As>
where
    T: Default,
    As: BitUnpackAs<T>,
{
    #[inline]
    fn unpack_as<R>(mut reader: R) -> Result<T, R::Error>
    where
        R: BitReader,
    {
        reader
            .unpack_as::<_, Option<As>>()
            .map(Option::unwrap_or_default)
    }
}
