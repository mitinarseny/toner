use ::bitvec::{order::Msb0, slice::BitSlice, store::BitStore, vec::BitVec};
use impl_tools::autoimpl;

use crate::{
    adapters::{BitCounter, MapErr, Tee},
    Error, ResultExt, StringError,
};

use super::{
    args::{r#as::BitPackAsWithArgs, BitPackWithArgs},
    r#as::BitPackAs,
    BitPack,
};

/// Bitwise writer.
#[autoimpl(for <W: trait + ?Sized> &mut W, Box<W>)]
pub trait BitWriter {
    // An error ocurred while writing
    type Error: Error;

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
        value.pack::<&mut Self>(self)?;
        Ok(self)
    }

    /// Pack given value with args using its [`BitPackWithArgs`] implementation
    #[inline]
    fn pack_with<T>(&mut self, value: T, args: T::Args) -> Result<&mut Self, Self::Error>
    where
        T: BitPackWithArgs,
    {
        value.pack_with::<&mut Self>(self, args)?;
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
        As::pack_as::<&mut Self>(&value, self)?;
        Ok(self)
    }

    /// Pack given value with args using an adapter.  
    /// See [`as`](crate::as) module-level documentation for more.
    #[inline]
    fn pack_as_with<T, As>(&mut self, value: T, args: As::Args) -> Result<&mut Self, Self::Error>
    where
        As: BitPackAsWithArgs<T> + ?Sized,
    {
        As::pack_as_with::<&mut Self>(&value, self, args)?;
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
        Tee {
            inner: self,
            writer,
        }
    }
}
impl<T> BitWriterExt for T where T: BitWriter {}

impl<W> BitWriter for BitCounter<W>
where
    W: BitWriter,
{
    type Error = W::Error;

    #[inline]
    fn write_bit(&mut self, bit: bool) -> Result<(), Self::Error> {
        self.inner.write_bit(bit)?;
        self.counter += 1;
        Ok(())
    }

    #[inline]
    fn write_bitslice(&mut self, bits: &BitSlice<u8, Msb0>) -> Result<(), Self::Error> {
        self.inner.write_bitslice(bits)?;
        self.counter += bits.len();
        Ok(())
    }

    #[inline]
    fn repeat_bit(&mut self, n: usize, bit: bool) -> Result<(), Self::Error> {
        self.inner.repeat_bit(n, bit)?;
        self.counter += n;
        Ok(())
    }
}

/// Adapter returned by [`.limit()`](BitWriterExt::limit)
#[autoimpl(Deref using self.inner)]
pub struct LimitWriter<W> {
    inner: BitCounter<W>,
    limit: usize,
}

impl<W> LimitWriter<W>
where
    W: BitWriter,
{
    #[inline]
    pub const fn new(writer: W, limit: usize) -> Self {
        Self {
            inner: BitCounter::new(writer),
            limit,
        }
    }

    #[inline]
    fn ensure_more(&self, n: usize) -> Result<(), W::Error> {
        if self.bit_count() + n > self.limit {
            return Err(Error::custom("max bits limit reached"));
        }
        Ok(())
    }

    #[inline]
    pub fn into_inner(self) -> W {
        self.inner.into_inner()
    }
}

impl<W> BitWriter for LimitWriter<W>
where
    W: BitWriter,
{
    type Error = W::Error;

    #[inline]
    fn write_bit(&mut self, bit: bool) -> Result<(), Self::Error> {
        self.ensure_more(1)?;
        self.inner.write_bit(bit)
    }

    #[inline]
    fn write_bitslice(&mut self, bits: &BitSlice<u8, Msb0>) -> Result<(), Self::Error> {
        self.ensure_more(bits.len())?;
        self.inner.write_bitslice(bits)
    }

    #[inline]
    fn repeat_bit(&mut self, n: usize, bit: bool) -> Result<(), Self::Error> {
        self.ensure_more(n)?;
        self.inner.repeat_bit(n, bit)
    }
}

impl<T, W> BitWriter for Tee<T, W>
where
    T: BitWriter,
    W: BitWriter,
{
    type Error = T::Error;

    #[inline]
    fn write_bit(&mut self, bit: bool) -> Result<(), Self::Error> {
        self.inner.write_bit(bit)?;
        self.writer
            .write_bit(bit)
            .map_err(<T::Error>::custom)
            .context("writer")
    }

    #[inline]
    fn write_bitslice(&mut self, bits: &BitSlice<u8, Msb0>) -> Result<(), Self::Error> {
        self.inner.write_bitslice(bits)?;
        self.writer
            .write_bitslice(bits)
            .map_err(<T::Error>::custom)
            .context("writer")
    }

    #[inline]
    fn repeat_bit(&mut self, n: usize, bit: bool) -> Result<(), Self::Error> {
        self.inner.repeat_bit(n, bit)?;
        self.writer
            .repeat_bit(n, bit)
            .map_err(<T::Error>::custom)
            .context("writer")
    }
}

impl<S> BitWriter for BitVec<S, Msb0>
where
    S: BitStore,
{
    type Error = StringError;

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

/// Binary string, e.g. `"0010110...."`
impl BitWriter for String {
    type Error = StringError;

    fn write_bit(&mut self, bit: bool) -> Result<(), Self::Error> {
        self.push(if bit { '1' } else { '0' });
        Ok(())
    }
}
