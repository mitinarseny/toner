use core::mem::size_of;
use std::fmt::Binary;

use crate::{BitPack, BitPackAs, Error, ResultExt, StringError};

use ::bitvec::{order::Msb0, slice::BitSlice, store::BitStore, vec::BitVec, view::AsBits};
use impl_tools::autoimpl;
use num_traits::{PrimInt, ToBytes};

#[autoimpl(for <W: trait + ?Sized> &mut W, Box<W>)]
pub trait BitWriter {
    type Error: Error;

    fn write_bit(&mut self, bit: bool) -> Result<(), Self::Error>;

    #[inline]
    fn write_bitslice(&mut self, bits: &BitSlice<u8, Msb0>) -> Result<(), Self::Error> {
        for bit in bits {
            self.write_bit(*bit)?;
        }
        Ok(())
    }

    #[inline]
    fn repeat_bit(&mut self, n: usize, bit: bool) -> Result<(), Self::Error> {
        for _ in 0..n {
            self.write_bit(bit)?;
        }
        Ok(())
    }
}

pub trait BitWriterExt: BitWriter {
    #[inline]
    fn with_bit(&mut self, bit: bool) -> Result<&mut Self, Self::Error> {
        self.write_bit(bit)?;
        Ok(self)
    }

    #[inline]
    fn with_bits(
        &mut self,
        bits: impl AsRef<BitSlice<u8, Msb0>>,
    ) -> Result<&mut Self, Self::Error> {
        self.write_bitslice(bits.as_ref())?;
        Ok(self)
    }

    #[inline]
    fn with_repeat_bit(&mut self, n: usize, bit: bool) -> Result<&mut Self, Self::Error> {
        self.repeat_bit(n, bit)?;
        Ok(self)
    }

    #[inline]
    fn with_bytes(&mut self, bytes: impl AsRef<[u8]>) -> Result<&mut Self, Self::Error> {
        self.with_bits(bytes.as_bits::<Msb0>())?;
        Ok(self)
    }

    #[inline]
    fn pack<T>(&mut self, value: T) -> Result<&mut Self, Self::Error>
    where
        T: BitPack,
    {
        value.pack::<&mut Self>(self)?;
        Ok(self)
    }

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

    #[inline]
    fn pack_as<T, As>(&mut self, value: T) -> Result<&mut Self, Self::Error>
    where
        As: BitPackAs<T> + ?Sized,
    {
        As::pack_as::<&mut Self>(&value, self)?;
        Ok(self)
    }

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

    #[inline]
    fn pack_as_n_bytes<T>(&mut self, value: T, num_bytes: u32) -> Result<&mut Self, Self::Error>
    where
        T: PrimInt + Binary + ToBytes,
    {
        let size_bytes: u32 = size_of::<T>() as u32;
        let leading_zeroes = value.leading_zeros();
        let used_bytes = size_bytes - (leading_zeroes + 7) / 8;
        if num_bytes < used_bytes {
            return Err(Error::custom(format!(
                "{value:0b} cannot be packed into {num_bytes} bytes",
            )));
        }
        let arr = value.to_be_bytes();
        let mut bytes = arr.as_ref();
        bytes = &bytes[bytes.len() - used_bytes as usize..];
        self.write_bitslice(bytes.as_bits())?;
        Ok(self)
    }

    #[inline]
    fn as_mut(&mut self) -> &mut Self {
        self
    }

    #[inline]
    fn counted(self) -> BitCounter<Self>
    where
        Self: Sized,
    {
        BitCounter::new(self)
    }

    #[inline]
    fn limit(self, n: usize) -> LimitWriter<Self>
    where
        Self: Sized,
    {
        LimitWriter::new(self, n)
    }
}

impl<T> BitWriterExt for T where T: BitWriter {}

#[autoimpl(Deref using self.inner)]
pub struct BitCounter<W> {
    inner: W,
    bits_written: usize,
}

impl<W> BitCounter<W> {
    #[inline]
    pub const fn new(writer: W) -> Self {
        Self {
            inner: writer,
            bits_written: 0,
        }
    }

    #[inline]
    pub const fn bits_written(&self) -> usize {
        self.bits_written
    }

    #[inline]
    pub fn into_inner(self) -> W {
        self.inner
    }
}

impl<W> BitWriter for BitCounter<W>
where
    W: BitWriter,
{
    type Error = W::Error;

    #[inline]
    fn write_bit(&mut self, bit: bool) -> Result<(), Self::Error> {
        self.inner.write_bit(bit)?;
        self.bits_written += 1;
        Ok(())
    }

    #[inline]
    fn write_bitslice(&mut self, bits: &BitSlice<u8, Msb0>) -> Result<(), Self::Error> {
        self.inner.write_bitslice(bits)?;
        self.bits_written += bits.len();
        Ok(())
    }

    #[inline]
    fn repeat_bit(&mut self, n: usize, bit: bool) -> Result<(), Self::Error> {
        self.inner.repeat_bit(n, bit)?;
        self.bits_written += n;
        Ok(())
    }
}

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
        if self.bits_written() + n > self.limit {
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
