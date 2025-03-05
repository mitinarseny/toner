use core::iter;

use ::bitvec::{order::Msb0, slice::BitSlice, view::AsMutBits};
use impl_tools::autoimpl;

use crate::{
    adapters::{Join, MapErr, Tee},
    ser::BitWriter,
    Error, ResultExt, StringError,
};

use super::{
    args::{r#as::BitUnpackAsWithArgs, BitUnpackWithArgs},
    r#as::BitUnpackAs,
    BitUnpack,
};

/// Bitwise reader.
#[autoimpl(for <R: trait + ?Sized> &mut R, Box<R>)]
pub trait BitReader {
    // An error ocurred while reading
    type Error: Error;

    /// Returns count of bits left to read more
    fn bits_left(&self) -> usize;

    /// Reads only one bit.
    fn read_bit(&mut self) -> Result<Option<bool>, Self::Error>;

    /// Reads `dst.len()` bits into given bitslice.
    /// Might be optimized by the implementation.
    #[inline]
    fn read_bits_into(&mut self, dst: &mut BitSlice<u8, Msb0>) -> Result<usize, Self::Error> {
        for (i, mut bit) in dst.iter_mut().enumerate() {
            let Some(read) = self.read_bit()? else {
                return Ok(i);
            };
            *bit = read;
        }
        Ok(dst.len())
    }

    /// Reads and discards `n` bits
    #[inline]
    fn skip(&mut self, n: usize) -> Result<usize, Self::Error> {
        for i in 0..n {
            if self.read_bit()?.is_none() {
                return Ok(i);
            }
        }
        Ok(n)
    }
}

/// Extension helper for [`BitReader`].
pub trait BitReaderExt: BitReader {
    /// Returns wheather the reader is empty
    #[inline]
    fn is_empty(&self) -> bool {
        self.bits_left() == 0
    }

    /// Reads `dst.len()` bytes into given byte slice
    #[inline]
    fn read_bytes_into(&mut self, mut dst: impl AsMut<[u8]>) -> Result<usize, Self::Error> {
        self.read_bits_into(dst.as_mut_bits())
    }

    /// Read `N` bytes and return array
    #[inline]
    fn read_bytes_array<const N: usize>(&mut self) -> Result<[u8; N], Self::Error> {
        let mut arr = [0; N];
        self.read_bits_into(arr.as_mut_bits())?;
        Ok(arr)
    }

    /// Unpack value using its [`BitUnpack`] implementation
    #[inline]
    fn unpack<T>(&mut self) -> Result<T, Self::Error>
    where
        T: BitUnpack,
    {
        T::unpack(self)
    }

    /// Unpack value witg args using its [`BitUnpackWithArgs`] implementation
    #[inline]
    fn unpack_with<T>(&mut self, args: T::Args) -> Result<T, Self::Error>
    where
        T: BitUnpackWithArgs,
    {
        T::unpack_with(self, args)
    }

    /// Return iterator that unpacks values using [`BitUnpack`] implementation
    #[inline]
    fn unpack_iter<T>(&mut self) -> impl Iterator<Item = Result<T, Self::Error>> + '_
    where
        T: BitUnpack,
    {
        iter::repeat_with(move || self.unpack::<T>())
            .enumerate()
            .map(|(i, v)| v.with_context(|| format!("[{i}]")))
    }

    /// Return iterator that unpacks values with args using [`BitUnpackWithArgs`] implementation
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

    /// Unpack value using an adapter.  
    /// See [`as`](crate::as) module-level documentation for more.
    #[inline]
    fn unpack_as<T, As>(&mut self) -> Result<T, Self::Error>
    where
        As: BitUnpackAs<T> + ?Sized,
    {
        As::unpack_as(self)
    }

    /// Unpack value with args using an adapter.  
    /// See [`as`](crate::as) module-level documentation for more.
    #[inline]
    fn unpack_as_with<T, As>(&mut self, args: As::Args) -> Result<T, Self::Error>
    where
        As: BitUnpackAsWithArgs<T> + ?Sized,
    {
        As::unpack_as_with(self, args)
    }

    /// Returns iterator that unpacks values using an adapter.  
    /// See [`as`](crate::as) module-level documentation for more.
    #[inline]
    fn unpack_iter_as<T, As>(&mut self) -> impl Iterator<Item = Result<T, Self::Error>> + '_
    where
        As: BitUnpackAs<T> + ?Sized,
    {
        iter::repeat_with(|| self.unpack_as::<_, As>())
            .enumerate()
            .map(|(i, v)| v.with_context(|| format!("[{i}]")))
    }

    /// Returns iterator that unpacks values with args using an adapter.  
    /// See [`as`](crate::as) module-level documentation for more.
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

    /// Borrows reader, rather than consuming it.
    #[inline]
    fn as_mut(&mut self) -> &mut Self {
        self
    }

    /// Map [`Error`](BitReader::Error) by given closure
    #[inline]
    fn map_err<F>(self, f: F) -> MapErr<Self, F>
    where
        Self: Sized,
    {
        MapErr { inner: self, f }
    }

    /// Mirror all read data to given writer as well.
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

    #[inline]
    fn join<R>(self, next: R) -> Join<Self, R>
    where
        Self: Sized,
        R: BitReader,
    {
        Join(self, next)
    }
}
impl<T> BitReaderExt for T where T: BitReader {}

impl<R, F, E> BitReader for MapErr<R, F>
where
    R: BitReader,
    F: FnMut(R::Error) -> E,
    E: Error,
{
    type Error = E;

    #[inline]
    fn bits_left(&self) -> usize {
        self.inner.bits_left()
    }

    #[inline]
    fn read_bit(&mut self) -> Result<Option<bool>, Self::Error> {
        self.inner.read_bit().map_err(&mut self.f)
    }

    #[inline]
    fn read_bits_into(&mut self, dst: &mut BitSlice<u8, Msb0>) -> Result<usize, Self::Error> {
        self.inner.read_bits_into(dst).map_err(&mut self.f)
    }

    #[inline]
    fn skip(&mut self, n: usize) -> Result<usize, Self::Error> {
        self.inner.skip(n).map_err(&mut self.f)
    }
}

impl<R, W> BitReader for Tee<R, W>
where
    R: BitReader,
    W: BitWriter,
{
    type Error = R::Error;

    #[inline]
    fn bits_left(&self) -> usize {
        self.inner.bits_left()
    }

    #[inline]
    fn read_bit(&mut self) -> Result<Option<bool>, Self::Error> {
        let Some(bit) = self.inner.read_bit()? else {
            return Ok(None);
        };
        self.writer
            .write_bit(bit)
            .map_err(<R::Error>::custom)
            .context("writer")?;
        Ok(Some(bit))
    }

    #[inline]
    fn read_bits_into(&mut self, dst: &mut BitSlice<u8, Msb0>) -> Result<usize, Self::Error> {
        let n = self.inner.read_bits_into(dst)?;
        self.writer
            .write_bitslice(&dst[..n])
            .map_err(|err| <R::Error>::custom(err).context("writer"))?;
        Ok(n)
    }
}

impl<R1, R2> BitReader for Join<R1, R2>
where
    R1: BitReader,
    R2: BitReader,
{
    type Error = R1::Error;

    #[inline]
    fn bits_left(&self) -> usize {
        self.0.bits_left() + self.1.bits_left()
    }

    #[inline]
    fn read_bit(&mut self) -> Result<Option<bool>, Self::Error> {
        if let Some(bit) = self.0.read_bit()? {
            return Ok(Some(bit));
        }
        self.1.read_bit().map_err(Error::custom)
    }

    #[inline]
    fn read_bits_into(&mut self, dst: &mut BitSlice<u8, Msb0>) -> Result<usize, Self::Error> {
        let n = self.0.read_bits_into(dst)?;
        Ok(n + self
            .1
            .read_bits_into(&mut dst[n..])
            .map_err(Error::custom)?)
    }

    #[inline]
    fn skip(&mut self, n: usize) -> Result<usize, Self::Error> {
        let skipped = self.0.skip(n)?;
        Ok(skipped + self.1.skip(n - skipped).map_err(Error::custom)?)
    }
}

impl BitReader for &BitSlice<u8, Msb0> {
    type Error = StringError;

    #[inline]
    fn bits_left(&self) -> usize {
        self.len()
    }

    #[inline]
    fn read_bit(&mut self) -> Result<Option<bool>, Self::Error> {
        let Some((bit, rest)) = self.split_first() else {
            return Ok(None);
        };
        *self = rest;
        Ok(Some(*bit))
    }

    #[inline]
    fn read_bits_into(&mut self, dst: &mut BitSlice<u8, Msb0>) -> Result<usize, Self::Error> {
        let n = dst.len().min(self.bits_left());
        let (v, rest) = self.split_at(n);
        dst[..n].copy_from_bitslice(v);
        *self = rest;
        Ok(n)
    }

    #[inline]
    fn skip(&mut self, mut n: usize) -> Result<usize, Self::Error> {
        n = n.min(self.bits_left());
        let (_, rest) = self.split_at(n);
        *self = rest;
        Ok(n)
    }
}
