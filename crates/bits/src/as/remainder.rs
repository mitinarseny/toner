use std::borrow::Cow;

use bitvec::{order::Msb0, slice::BitSlice, vec::BitVec};

use crate::{
    r#as::BorrowCow,
    de::{BitReader, BitReaderExt, r#as::BitUnpackAs},
};

pub struct Remainder;

impl<'de: 'a, 'a> BitUnpackAs<'de, Cow<'a, BitSlice<u8, Msb0>>> for Remainder {
    #[inline]
    fn unpack_as<R>(mut reader: R) -> Result<Cow<'a, BitSlice<u8, Msb0>>, R::Error>
    where
        R: BitReader<'de>,
    {
        let n = reader.bits_left();
        reader.unpack_as_with::<_, BorrowCow>(n)
    }
}

impl<'de> BitUnpackAs<'de, BitVec<u8, Msb0>> for Remainder {
    #[inline]
    fn unpack_as<R>(mut reader: R) -> Result<BitVec<u8, Msb0>, R::Error>
    where
        R: BitReader<'de>,
    {
        reader
            .unpack_as::<Cow<BitSlice<u8, Msb0>>, Self>()
            .map(Cow::into_owned)
    }
}

impl<'de: 'a, 'a> BitUnpackAs<'de, Cow<'a, [u8]>> for Remainder {
    #[inline]
    fn unpack_as<R>(mut reader: R) -> Result<Cow<'a, [u8]>, R::Error>
    where
        R: BitReader<'de>,
    {
        let n = reader.bits_left();
        reader.unpack_as_with::<_, BorrowCow>(n)
    }
}

impl<'de> BitUnpackAs<'de, Vec<u8>> for Remainder {
    #[inline]
    fn unpack_as<R>(mut reader: R) -> Result<Vec<u8>, R::Error>
    where
        R: BitReader<'de>,
    {
        reader.unpack_as::<Cow<[u8]>, Self>().map(Cow::into_owned)
    }
}

impl<'de: 'a, 'a> BitUnpackAs<'de, Cow<'a, str>> for Remainder {
    #[inline]
    fn unpack_as<R>(mut reader: R) -> Result<Cow<'a, str>, R::Error>
    where
        R: BitReader<'de>,
    {
        let n = reader.bits_left();
        reader.unpack_as_with::<_, BorrowCow>(n)
    }
}

impl<'de> BitUnpackAs<'de, String> for Remainder {
    #[inline]
    fn unpack_as<R>(mut reader: R) -> Result<String, R::Error>
    where
        R: BitReader<'de>,
    {
        reader.unpack_as::<Cow<str>, Self>().map(Cow::into_owned)
    }
}
