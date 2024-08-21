use bitvec::{order::Msb0, vec::BitVec};

use crate::{
    de::{r#as::BitUnpackAs, BitReader, BitReaderExt},
    Error,
};

pub struct Remainder;

impl BitUnpackAs<BitVec<u8, Msb0>> for Remainder {
    #[inline]
    fn unpack_as<R>(mut reader: R) -> Result<BitVec<u8, Msb0>, R::Error>
    where
        R: BitReader,
    {
        let n = reader.bits_left();
        reader.unpack_with(n)
    }
}

impl BitUnpackAs<Vec<u8>> for Remainder {
    #[inline]
    fn unpack_as<R>(mut reader: R) -> Result<Vec<u8>, R::Error>
    where
        R: BitReader,
    {
        let bits: BitVec<u8, Msb0> = reader.unpack_as::<_, Self>()?;
        if bits.len() % 8 != 0 {
            return Err(Error::custom("EOF"));
        }
        Ok(bits.into_vec())
    }
}

impl BitUnpackAs<String> for Remainder {
    #[inline]
    fn unpack_as<R>(mut reader: R) -> Result<String, R::Error>
    where
        R: BitReader,
    {
        let bytes: Vec<u8> = reader.unpack_as::<_, Self>()?;
        String::from_utf8(bytes).map_err(Error::custom)
    }
}
