use core::{iter, mem::size_of};

use ::bitvec::{order::Msb0, slice::BitSlice, vec::BitVec, view::AsMutBits};
use impl_tools::autoimpl;
use num_traits::PrimInt;

use crate::{BitUnpack, BitUnpackAs, BitWriter, Error, MapErr, ResultExt, StringError};

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
    fn unpack_iter<T>(&mut self) -> impl Iterator<Item = Result<T, Self::Error>> + '_
    where
        T: BitUnpack,
    {
        iter::repeat_with(move || self.unpack::<T>())
            .enumerate()
            .map(|(i, v)| v.with_context(|| format!("[{i}]")))
    }

    #[inline]
    fn unpack_as<T, As>(&mut self) -> Result<T, Self::Error>
    where
        As: BitUnpackAs<T> + ?Sized,
    {
        As::unpack_as(self)
    }

    #[inline]
    fn unpack_iter_as<T, As>(&mut self) -> impl Iterator<Item = Result<T, Self::Error>> + '_
    where
        As: BitUnpackAs<T> + ?Sized,
    {
        iter::repeat_with(|| self.unpack_as::<_, As>())
            .enumerate()
            .map(|(i, v)| v.with_context(|| format!("[{i}]")))
    }

    #[inline]
    fn unpack_as_n_bytes<T>(&mut self, num_bytes: u32) -> Result<T, Self::Error>
    where
        T: PrimInt,
    {
        let size_bytes: u32 = size_of::<T>() as u32;
        if num_bytes > size_bytes {
            return Err(Error::custom("excessive bits for type"));
        }
        let mut v: T = T::zero();
        for byte in self.unpack_iter::<u8>().take(num_bytes as usize) {
            v = v << 8;
            v = v | T::from(byte?).unwrap();
        }
        Ok(v)
    }

    fn map_err<F>(self, f: F) -> MapErr<Self, F>
    where
        Self: Sized,
    {
        MapErr { inner: self, f }
    }

    fn tee<W>(self, writer: W) -> TeeReader<Self, W>
    where
        Self: Sized,
        W: BitWriter,
    {
        TeeReader {
            inner: self,
            writer,
        }
    }
}

impl<T> BitReaderExt for T where T: BitReader {}

pub struct TeeReader<R, W> {
    inner: R,
    writer: W,
}

impl<R, W> TeeReader<R, W> {
    pub fn into_inner(self) -> R {
        self.inner
    }
}

impl<R, W> BitReader for TeeReader<R, W>
where
    R: BitReader,
    W: BitWriter,
{
    type Error = R::Error;

    fn read_bit(&mut self) -> Result<bool, Self::Error> {
        let bit = self.inner.read_bit()?;
        self.writer
            .write_bit(bit)
            .map_err(<R::Error>::custom)
            .context("writer")?;
        Ok(bit)
    }

    #[inline]
    fn read_bits_into(&mut self, dst: &mut BitSlice<u8, Msb0>) -> Result<(), Self::Error> {
        self.inner.read_bits_into(dst)?;
        self.writer
            .write_bitslice(dst)
            .map_err(|err| <R::Error>::custom(err).context("writer"))?;
        Ok(())
    }
}

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
