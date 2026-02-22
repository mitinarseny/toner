use std::mem;

use ::bitvec::{order::Msb0, slice::BitSlice, store::BitStore, vec::BitVec};

use impl_tools::autoimpl;

use crate::{
    Context, Error, StringError,
    adapters::{BitCounter, LimitWriter, MapErr, Tee},
};

use super::{
    BitPack,
    args::{BitPackWithArgs, r#as::BitPackAsWithArgs},
    r#as::BitPackAs,
};

/// Bitwise writer.
#[autoimpl(for <W: trait + ?Sized> &mut W, Box<W>)]
pub trait BitWriter {
    // An error ocurred while writing
    type Error: Error;

    /// Returns remaining capacity in bits
    fn capacity_left(&self) -> usize;

    /// Writes a single bit.
    fn write_bit(&mut self, bit: bool) -> Result<(), Self::Error>;

    /// Writes given bitslice.  
    /// Might be optimized by the implementation.
    #[inline]
    fn write_bitslice(&mut self, bits: &BitSlice<u8, Msb0>) -> Result<(), Self::Error> {
        for bit in bits {
            self.write_bit(*bit)?;
        }
        Ok(())
    }

    /// Writes given `bit` exactly `n` times.  
    /// Might be optimized by the implementation.
    #[inline]
    fn repeat_bit(&mut self, n: usize, bit: bool) -> Result<(), Self::Error> {
        for _ in 0..n {
            self.write_bit(bit)?;
        }
        Ok(())
    }
}

/// Extension helper for [`BitWriter`].
pub trait BitWriterExt: BitWriter {
    /// Same as [`.repeat_bit()`](BitWriter::repeat_bit) but can be used
    /// for chaining
    #[inline]
    fn with_repeat_bit(&mut self, n: usize, bit: bool) -> Result<&mut Self, Self::Error> {
        self.repeat_bit(n, bit)?;
        Ok(self)
    }

    /// Pack given value using its [`BitPack`] implementation
    #[inline]
    fn pack<T>(&mut self, value: T) -> Result<&mut Self, Self::Error>
    where
        T: BitPack,
    {
        value.pack(self)?;
        Ok(self)
    }

    /// Pack given value with args using its [`BitPackWithArgs`] implementation
    #[inline]
    fn pack_with<T>(&mut self, value: T, args: T::Args) -> Result<&mut Self, Self::Error>
    where
        T: BitPackWithArgs,
    {
        value.pack_with(self, args)?;
        Ok(self)
    }

    /// Pack all values from given iterator using [`BitPack`] implementation
    /// of its item type.
    #[inline]
    fn pack_many<T>(
        &mut self,
        values: impl IntoIterator<Item = T>,
    ) -> Result<&mut Self, Self::Error>
    where
        T: BitPack,
    {
        for (i, v) in values.into_iter().enumerate() {
            self.pack(v).with_context(|| format!("[{i}]"))?;
        }
        Ok(self)
    }

    /// Pack all values with args from given iterator using [`BitPackWithArgs`]
    /// implementation of its item type.
    #[inline]
    fn pack_many_with<T>(
        &mut self,
        values: impl IntoIterator<Item = T>,
        args: T::Args,
    ) -> Result<&mut Self, Self::Error>
    where
        T: BitPackWithArgs,
        T::Args: Clone,
    {
        for (i, v) in values.into_iter().enumerate() {
            self.pack_with(v, args.clone())
                .with_context(|| format!("[{i}]"))?;
        }
        Ok(self)
    }

    /// Pack given value using an adapter.  
    /// See [`as`](crate::as) module-level documentation for more.
    #[inline]
    fn pack_as<T, As>(&mut self, value: T) -> Result<&mut Self, Self::Error>
    where
        As: BitPackAs<T> + ?Sized,
    {
        As::pack_as(&value, self)?;
        Ok(self)
    }

    /// Pack given value with args using an adapter.  
    /// See [`as`](crate::as) module-level documentation for more.
    #[inline]
    fn pack_as_with<T, As>(&mut self, value: T, args: As::Args) -> Result<&mut Self, Self::Error>
    where
        As: BitPackAsWithArgs<T> + ?Sized,
    {
        As::pack_as_with(&value, self, args)?;
        Ok(self)
    }

    /// Pack all values from iterator using an adapter.  
    /// See [`as`](crate::as) module-level documentation for more.
    #[inline]
    fn pack_many_as<T, As>(
        &mut self,
        values: impl IntoIterator<Item = T>,
    ) -> Result<&mut Self, Self::Error>
    where
        As: BitPackAs<T> + ?Sized,
    {
        for (i, v) in values.into_iter().enumerate() {
            self.pack_as::<_, As>(v).with_context(|| format!("[{i}]"))?;
        }
        Ok(self)
    }

    /// Pack all values from iterator with args using an adapter.  
    /// See [`as`](crate::as) module-level documentation for more.
    #[inline]
    fn pack_many_as_with<T, As>(
        &mut self,
        values: impl IntoIterator<Item = T>,
        args: As::Args,
    ) -> Result<&mut Self, Self::Error>
    where
        As: BitPackAsWithArgs<T> + ?Sized,
        As::Args: Clone,
    {
        for (i, v) in values.into_iter().enumerate() {
            self.pack_as_with::<_, As>(v, args.clone())
                .with_context(|| format!("[{i}]"))?;
        }
        Ok(self)
    }

    /// Borrows writer, rather than consuming it.
    #[inline]
    fn as_mut(&mut self) -> &mut Self {
        self
    }

    /// Map [`Error`](BitWriter::Error) by given closure
    #[inline]
    fn map_err<F>(self, f: F) -> MapErr<Self, F>
    where
        Self: Sized,
    {
        MapErr { inner: self, f }
    }

    /// Wrap this writer to count written bits by using
    /// [`.bit_count()`](BitCounter::bit_count).
    #[inline]
    fn counted(self) -> BitCounter<Self>
    where
        Self: Sized,
    {
        BitCounter::new(self)
    }

    /// Sets given limit on this writer.  
    /// Returned wrapped writer will return an error when caller tries to
    /// write value which will exceed the total limit by using
    /// [`.pack()`](BitWriterExt::pack) or any similar method.
    #[inline]
    fn limit(self, n: usize) -> LimitWriter<Self>
    where
        Self: Sized,
    {
        LimitWriter::new(self, n)
    }

    /// Mirror all written data to given writer as well.
    #[inline]
    fn tee<W>(self, writer: W) -> Tee<Self, W>
    where
        Self: Sized,
        W: BitWriter,
    {
        Tee::new(self, writer)
    }
}
impl<T> BitWriterExt for T where T: BitWriter + ?Sized {}

#[derive(Debug, Clone, Copy)]
pub struct NoopBitWriter;

impl BitWriter for NoopBitWriter {
    type Error = StringError;

    #[inline]
    fn capacity_left(&self) -> usize {
        usize::MAX
    }

    #[inline]
    fn write_bit(&mut self, _bit: bool) -> Result<(), Self::Error> {
        Ok(())
    }

    #[inline]
    fn write_bitslice(&mut self, _bits: &BitSlice<u8, Msb0>) -> Result<(), Self::Error> {
        Ok(())
    }

    #[inline]
    fn repeat_bit(&mut self, _n: usize, _bit: bool) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl BitWriter for &mut BitSlice<u8, Msb0> {
    type Error = StringError;

    #[inline]
    fn capacity_left(&self) -> usize {
        self.len()
    }

    #[inline]
    fn write_bit(&mut self, bit: bool) -> Result<(), Self::Error> {
        if self.is_empty() {
            return Err(Error::custom("EOF"));
        }
        *self = unsafe {
            *self.get_unchecked_mut(0) = bit;
            mem::take(self).get_unchecked_mut(1..)
        };
        Ok(())
    }

    #[inline]
    fn write_bitslice(&mut self, bits: &BitSlice<u8, Msb0>) -> Result<(), Self::Error> {
        if self.capacity_left() < bits.len() {
            return Err(Error::custom("EOF"));
        }
        *self = unsafe {
            self.get_unchecked_mut(..bits.len())
                .copy_from_bitslice(bits);
            mem::take(self).get_unchecked_mut(bits.len()..)
        };
        Ok(())
    }

    #[inline]
    fn repeat_bit(&mut self, n: usize, bit: bool) -> Result<(), Self::Error> {
        if self.capacity_left() < n {
            return Err(Error::custom("EOF"));
        }
        *self = unsafe {
            self.get_unchecked_mut(..n).fill(bit);
            mem::take(self).get_unchecked_mut(n..)
        };
        Ok(())
    }
}

impl<S> BitWriter for BitVec<S, Msb0>
where
    S: BitStore,
{
    type Error = StringError;

    #[inline]
    fn capacity_left(&self) -> usize {
        usize::MAX - self.len()
    }

    #[inline]
    fn write_bit(&mut self, bit: bool) -> Result<(), Self::Error> {
        self.push(bit);
        Ok(())
    }

    #[inline]
    fn write_bitslice(&mut self, bits: &BitSlice<u8, Msb0>) -> Result<(), Self::Error> {
        self.extend_from_bitslice(bits);
        Ok(())
    }

    #[inline]
    fn repeat_bit(&mut self, n: usize, bit: bool) -> Result<(), Self::Error> {
        self.resize(self.len() + n, bit);
        Ok(())
    }
}

impl BitWriter for Vec<bool> {
    type Error = StringError;

    #[inline]
    fn capacity_left(&self) -> usize {
        usize::MAX - self.len()
    }

    #[inline]
    fn write_bit(&mut self, bit: bool) -> Result<(), Self::Error> {
        self.push(bit);
        Ok(())
    }
}

/// Binary string, e.g. `"0010110...."`
impl BitWriter for String {
    type Error = StringError;

    #[inline]
    fn capacity_left(&self) -> usize {
        usize::MAX - self.len()
    }

    #[inline]
    fn write_bit(&mut self, bit: bool) -> Result<(), Self::Error> {
        self.push(if bit { '1' } else { '0' });
        Ok(())
    }
}
