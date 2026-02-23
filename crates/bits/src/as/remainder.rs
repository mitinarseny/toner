use std::borrow::Cow;

use bitvec::{order::Msb0, slice::BitSlice, vec::BitVec};

use crate::{
    r#as::BorrowCow,
    de::{BitReader, BitReaderExt, BitUnpackAs},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Remainder;

impl<'de: 'a, 'a> BitUnpackAs<'de, Cow<'a, BitSlice<u8, Msb0>>> for Remainder {
    type Args = ();

    #[inline]
    fn unpack_as<R>(reader: &mut R, _: Self::Args) -> Result<Cow<'a, BitSlice<u8, Msb0>>, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        let n = reader.bits_left();
        reader.unpack_as::<_, BorrowCow>(n)
    }
}

impl<'de> BitUnpackAs<'de, BitVec<u8, Msb0>> for Remainder {
    type Args = ();

    #[inline]
    fn unpack_as<R>(reader: &mut R, _: Self::Args) -> Result<BitVec<u8, Msb0>, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        reader
            .unpack_as::<Cow<BitSlice<u8, Msb0>>, Self>(())
            .map(Cow::into_owned)
    }
}

impl<'de: 'a, 'a> BitUnpackAs<'de, Cow<'a, [u8]>> for Remainder {
    type Args = ();

    #[inline]
    fn unpack_as<R>(reader: &mut R, _: Self::Args) -> Result<Cow<'a, [u8]>, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        let n = reader.bits_left();
        reader.unpack_as::<_, BorrowCow>(n)
    }
}

impl<'de> BitUnpackAs<'de, Vec<u8>> for Remainder {
    type Args = ();

    #[inline]
    fn unpack_as<R>(reader: &mut R, _: Self::Args) -> Result<Vec<u8>, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        reader.unpack_as::<Cow<[u8]>, Self>(()).map(Cow::into_owned)
    }
}

impl<'de: 'a, 'a> BitUnpackAs<'de, Cow<'a, str>> for Remainder {
    type Args = ();

    #[inline]
    fn unpack_as<R>(reader: &mut R, _: Self::Args) -> Result<Cow<'a, str>, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        let n = reader.bits_left();
        reader.unpack_as::<_, BorrowCow>(n)
    }
}

impl<'de> BitUnpackAs<'de, String> for Remainder {
    type Args = ();

    #[inline]
    fn unpack_as<R>(reader: &mut R, _: Self::Args) -> Result<String, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        reader.unpack_as::<Cow<str>, Self>(()).map(Cow::into_owned)
    }
}
