use std::borrow::Cow;

use bitvec::{mem::bits_of, order::Msb0, slice::BitSlice};

use crate::{
    Error,
    de::{BitReader, BitReaderExt, BitUnpackAs},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BorrowCow;

impl<'de: 'a, 'a> BitUnpackAs<'de, Cow<'a, BitSlice<u8, Msb0>>> for BorrowCow {
    /// length in bits
    type Args = usize;

    #[inline]
    fn unpack_as<R>(
        reader: &mut R,
        len: Self::Args,
    ) -> Result<Cow<'a, BitSlice<u8, Msb0>>, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        let v = reader.read_bits(len)?;
        if v.len() != len {
            return Err(Error::custom("EOF"));
        }
        Ok(v)
    }
}

impl<'de: 'a, 'a> BitUnpackAs<'de, Cow<'a, [u8]>> for BorrowCow {
    /// length in bytes
    type Args = usize;

    #[inline]
    fn unpack_as<R>(reader: &mut R, len: Self::Args) -> Result<Cow<'a, [u8]>, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        let len_bits = len * bits_of::<u8>();
        let v = reader.read_bits(len_bits)?;
        if v.len() != len_bits {
            return Err(Error::custom("EOF"));
        }
        if let Cow::Borrowed(s) = v {
            if let Some((head, body, tail)) = s.domain().region() {
                if head.is_none() && tail.is_none() {
                    return Ok(Cow::Borrowed(body));
                }
            }
        }

        let mut v = v.into_owned();
        // BitVec might not start from the first element after ToOwned
        v.force_align();
        Ok(Cow::Owned(v.into_vec()))
    }
}

impl<'de: 'a, 'a> BitUnpackAs<'de, Cow<'a, str>> for BorrowCow {
    /// length in bytes
    type Args = usize;

    #[rustversion::before(1.87)]
    #[inline]
    fn unpack_as<R>(reader: &mut R, len: Self::Args) -> Result<Cow<'a, str>, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        let bytes: Vec<u8> = reader.unpack((len, ()))?;
        String::from_utf8(bytes)
            .map(Cow::Owned)
            .map_err(Error::custom)
    }

    #[rustversion::since(1.87)]
    #[inline]
    fn unpack_as<R>(reader: &mut R, len: Self::Args) -> Result<Cow<'a, str>, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        match reader.unpack_as::<Cow<[u8]>, Self>(len)? {
            Cow::Borrowed(s) =>
            {
                #[allow(clippy::incompatible_msrv)]
                str::from_utf8(s).map(Cow::Borrowed).map_err(Error::custom)
            }
            Cow::Owned(v) => String::from_utf8(v).map(Cow::Owned).map_err(Error::custom),
        }
    }
}
