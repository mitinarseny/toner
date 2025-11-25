use core::iter;
use std::borrow::Cow;

use bitvec::{mem::bits_of, order::Msb0, slice::BitSlice, vec::BitVec, view::AsMutBits};
use impl_tools::autoimpl;

use crate::{
    Context, Error, StringError,
    adapters::{Checkpoint, Join, MapErr, Tee},
    ser::BitWriter,
};

use super::{
    BitUnpack,
    args::{BitUnpackWithArgs, r#as::BitUnpackAsWithArgs},
    r#as::BitUnpackAs,
};

/// Bitwise reader.
#[autoimpl(for <R: trait + ?Sized> &mut R, Box<R>)]
pub trait BitReader<'de> {
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

    /// Reads `n` bits and returns possibly borrowed [`BitSlice`]
    #[inline]
    fn read_bits(&mut self, mut n: usize) -> Result<Cow<'de, BitSlice<u8, Msb0>>, Self::Error> {
        let mut buf = BitVec::repeat(false, n);
        n = self.read_bits_into(&mut buf)?;
        buf.truncate(n);
        Ok(Cow::Owned(buf))
    }

    /// Reads and discards `n` bits
    #[inline]
    fn skip(&mut self, n: usize) -> Result<usize, Self::Error> {
        for i in 1..=n {
            if self.read_bit()?.is_none() {
                return Ok(i);
            }
        }
        Ok(n)
    }
}

/// Extension helper for [`BitReader`].
pub trait BitReaderExt<'de>: BitReader<'de> {
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
        let n = self.read_bits_into(arr.as_mut_bits())?;
        if n != N * bits_of::<u8>() {
            return Err(Error::custom("EOF"));
        }
        Ok(arr)
    }

    /// Unpack value using its [`BitUnpack`] implementation
    #[inline]
    fn unpack<T>(&mut self) -> Result<T, Self::Error>
    where
        T: BitUnpack<'de>,
    {
        T::unpack(self)
    }

    /// Unpack value witg args using its [`BitUnpackWithArgs`] implementation
    #[inline]
    fn unpack_with<T>(&mut self, args: T::Args) -> Result<T, Self::Error>
    where
        T: BitUnpackWithArgs<'de>,
    {
        T::unpack_with(self, args)
    }

    /// Return iterator that unpacks values using [`BitUnpack`] implementation
    #[inline]
    fn unpack_iter<T>(&mut self) -> impl Iterator<Item = Result<T, Self::Error>> + '_
    where
        T: BitUnpack<'de>,
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
        T: BitUnpackWithArgs<'de>,
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
        As: BitUnpackAs<'de, T> + ?Sized,
    {
        As::unpack_as(self)
    }

    /// Unpack value with args using an adapter.  
    /// See [`as`](crate::as) module-level documentation for more.
    #[inline]
    fn unpack_as_with<T, As>(&mut self, args: As::Args) -> Result<T, Self::Error>
    where
        As: BitUnpackAsWithArgs<'de, T> + ?Sized,
    {
        As::unpack_as_with(self, args)
    }

    /// Returns iterator that unpacks values using an adapter.  
    /// See [`as`](crate::as) module-level documentation for more.
    #[inline]
    fn unpack_iter_as<T, As>(&mut self) -> impl Iterator<Item = Result<T, Self::Error>> + '_
    where
        As: BitUnpackAs<'de, T> + ?Sized,
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
        As: BitUnpackAsWithArgs<'de, T> + ?Sized,
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
        Tee::new(self, writer)
    }

    #[inline]
    fn checkpoint(self) -> Checkpoint<Self>
    where
        Self: Sized,
    {
        Checkpoint::new(self)
    }

    #[inline]
    fn join<R>(self, next: R) -> Join<Self, R>
    where
        Self: Sized,
        R: BitReader<'de>,
    {
        Join::new(self, next)
    }
}
impl<'de, T> BitReaderExt<'de> for T where T: BitReader<'de> {}

impl<'de> BitReader<'de> for &'de BitSlice<u8, Msb0> {
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
    fn read_bits(&mut self, n: usize) -> Result<Cow<'de, BitSlice<u8, Msb0>>, Self::Error> {
        let (v, rest) = self.split_at(n.min(self.bits_left()));
        *self = rest;
        Ok(Cow::Borrowed(v))
    }

    #[inline]
    fn skip(&mut self, mut n: usize) -> Result<usize, Self::Error> {
        n = n.min(self.bits_left());
        let (_, rest) = self.split_at(n);
        *self = rest;
        Ok(n)
    }
}

impl<'de> BitReader<'de> for &[bool] {
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
    fn skip(&mut self, mut n: usize) -> Result<usize, Self::Error> {
        n = n.min(self.bits_left());
        let (_, rest) = self.split_at(n);
        *self = rest;
        Ok(n)
    }
}

/// Binary string, e.g. `"0010110...."`
impl<'de> BitReader<'de> for &str {
    type Error = StringError;

    #[inline]
    fn bits_left(&self) -> usize {
        self.len()
    }

    #[inline]
    fn read_bit(&mut self) -> Result<Option<bool>, Self::Error> {
        let Some((char, rest)) = self.split_at_checked(1) else {
            return Ok(None);
        };
        let bit = match char {
            "0" => false,
            "1" => true,
            _ => {
                return Err(Error::custom(format!(
                    "invalid character: expected '0' or '1', got: {char}",
                )));
            }
        };
        *self = rest;
        Ok(Some(bit))
    }
}
