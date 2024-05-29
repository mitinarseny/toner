use core::iter;

use ::bitvec::{order::Msb0, slice::BitSlice, vec::BitVec, view::AsMutBits};
use impl_tools::autoimpl;

use crate::{
    BitUnpack, BitUnpackAs, BitUnpackAsWithArgs, BitUnpackWithArgs, BitWriter, Error, MapErr,
    ResultExt, StringError, Tee,
};

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
    fn unpack_with<T>(&mut self, args: T::Args) -> Result<T, Self::Error>
    where
        T: BitUnpackWithArgs,
    {
        T::unpack_with(self, args)
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
    fn unpack_iter_with<'a, T>(
        &'a mut self,
        args: T::Args,
    ) -> impl Iterator<Item = Result<T, Self::Error>> + 'a
    where
        T: BitUnpackWithArgs,
        T::Args: Clone + 'a,
    {
        iter::repeat_with(move || self.unpack_with::<T>(args.clone()))
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
    fn unpack_as_with<T, As>(&mut self, args: As::Args) -> Result<T, Self::Error>
    where
        As: BitUnpackAsWithArgs<T> + ?Sized,
    {
        As::unpack_as_with(self, args)
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
    fn unpack_iter_as_with<'a, T, As>(
        &'a mut self,
        args: As::Args,
    ) -> impl Iterator<Item = Result<T, Self::Error>> + 'a
    where
        As: BitUnpackAsWithArgs<T> + ?Sized,
        As::Args: Clone + 'a,
    {
        iter::repeat_with(move || self.unpack_as_with::<_, As>(args.clone()))
            .enumerate()
            .map(|(i, v)| v.with_context(|| format!("[{i}]")))
    }

    #[inline]
    fn as_mut(&mut self) -> &mut Self {
        self
    }

    #[inline]
    fn map_err<F>(self, f: F) -> MapErr<Self, F>
    where
        Self: Sized,
    {
        MapErr { inner: self, f }
    }

    #[inline]
    fn tee<W>(self, writer: W) -> Tee<Self, W>
    where
        Self: Sized,
        W: BitWriter,
    {
        Tee {
            inner: self,
            writer,
        }
    }
}

impl<R, F, E> BitReader for MapErr<R, F>
where
    R: BitReader,
    F: FnMut(R::Error) -> E,
    E: Error,
{
    type Error = E;

    fn read_bit(&mut self) -> Result<bool, Self::Error> {
        self.inner.read_bit().map_err(&mut self.f)
    }

    #[inline]
    fn read_bits_into(&mut self, dst: &mut BitSlice<u8, Msb0>) -> Result<(), Self::Error> {
        self.inner.read_bits_into(dst).map_err(&mut self.f)
    }

    #[inline]
    fn skip(&mut self, n: usize) -> Result<(), Self::Error> {
        self.inner.skip(n).map_err(&mut self.f)
    }
}

impl<T> BitReaderExt for T where T: BitReader {}

impl<R, W> BitReader for Tee<R, W>
where
    R: BitReader,
    W: BitWriter,
{
    type Error = R::Error;

    #[inline]
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
