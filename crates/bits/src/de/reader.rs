use ::bitvec::{order::Msb0, slice::BitSlice, vec::BitVec, view::AsMutBits};
use impl_tools::autoimpl;

use crate::{BitUnpack, BitUnpackAs, Error, StringError};

#[autoimpl(for <R: trait + ?Sized> &mut R, Box<R>)]
pub trait BitReader {
    type Error: Error;

    fn read_bit(&mut self) -> Result<bool, Self::Error>;

    #[inline]
    fn read_bits_into(&mut self, dst: &mut BitSlice<u8, Msb0>) -> Result<(), Self::Error> {
        for mut bit in dst.iter_mut() {
            *bit = self.read_bit()?
        }
        Ok(())
    }

    #[inline]
    fn skip(&mut self, n: usize) -> Result<(), Self::Error> {
        for _ in 0..n {
            self.read_bit()?;
        }
        Ok(())
    }
}

pub trait BitReaderExt: BitReader {
    #[inline]
    fn read_bitvec(&mut self, n: usize) -> Result<BitVec<u8, Msb0>, Self::Error> {
        let mut dst = BitVec::with_capacity(n);
        dst.resize(n, false);
        self.read_bits_into(&mut dst)?;
        Ok(dst)
    }

    #[inline]
    fn read_bytes_into(&mut self, mut dst: impl AsMut<[u8]>) -> Result<(), Self::Error> {
        self.read_bits_into(dst.as_mut_bits())
    }

    #[inline]
    fn read_bytes_array<const N: usize>(&mut self) -> Result<[u8; N], Self::Error> {
        let mut arr = [0; N];
        self.read_bits_into(arr.as_mut_bits())?;
        Ok(arr)
    }

    #[inline]
    fn read_bytes_vec(&mut self, n: usize) -> Result<Vec<u8>, Self::Error> {
        let mut v = vec![0; n];
        self.read_bytes_into(&mut v)?;
        Ok(v)
    }

    #[inline]
    fn unpack<T>(&mut self) -> Result<T, Self::Error>
    where
        T: BitUnpack,
    {
        T::unpack(self)
    }

    #[inline]
    fn unpack_as<T, As>(&mut self) -> Result<T, Self::Error>
    where
        As: BitUnpackAs<T> + ?Sized,
    {
        As::unpack_as(self)
    }
}

impl<T> BitReaderExt for T where T: BitReader {}

impl BitReader for &BitSlice<u8, Msb0> {
    type Error = StringError;

    #[inline]
    fn read_bit(&mut self) -> Result<bool, Self::Error> {
        let (bit, rest) = self.split_first().ok_or_else(|| Error::custom("EOF"))?;
        *self = rest;
        Ok(*bit)
    }

    #[inline]
    fn read_bits_into(&mut self, dst: &mut BitSlice<u8, Msb0>) -> Result<(), Self::Error> {
        if self.len() < dst.len() {
            return Err(Error::custom("EOF"));
        }
        let (v, rest) = self.split_at(dst.len());
        dst.copy_from_bitslice(v);
        *self = rest;
        Ok(())
    }

    #[inline]
    fn skip(&mut self, n: usize) -> Result<(), Self::Error> {
        if self.len() < n {
            return Err(Error::custom("EOF"));
        }
        let (_, rest) = self.split_at(n);
        *self = rest;
        Ok(())
    }
}
