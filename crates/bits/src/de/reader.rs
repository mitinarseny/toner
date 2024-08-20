use core::iter;

use ::bitvec::{order::Msb0, slice::BitSlice, view::AsMutBits};
use impl_tools::autoimpl;

use crate::{
    adapters::{MapErr, Tee},
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

    /// Reads only one bit.
    fn read_bit(&mut self) -> Result<bool, Self::Error>;

    /// Reads `dst.len()` bits into given bitslice.
    /// Might be optimized by the implementation.
    #[inline]
    fn read_bits_into(&mut self, dst: &mut BitSlice<u8, Msb0>) -> Result<(), Self::Error> {
        for mut bit in dst.iter_mut() {
            *bit = self.read_bit()?
        }
        Ok(())
    }

    /// Reads and discards `n` bits
    #[inline]
    fn skip(&mut self, n: usize) -> Result<(), Self::Error> {
        for _ in 0..n {
            self.read_bit()?;
        }
        Ok(())
    }
}

/// Extension helper for [`BitReader`].
pub trait BitReaderExt: BitReader {
    /// Reads `dst.len()` bytes into given byte slice
    #[inline]
    fn read_bytes_into(&mut self, mut dst: impl AsMut<[u8]>) -> Result<(), Self::Error> {
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
}
impl<T> BitReaderExt for T where T: BitReader {}

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
