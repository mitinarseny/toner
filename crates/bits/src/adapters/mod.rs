//! Adapters for [`BitReader`]/[`BitWriter`]
mod io;

use std::borrow::Cow;

use crate::{
    Context, Error,
    de::{BitReader, BitReaderExt},
    ser::BitWriter,
};

pub use self::io::*;

use bitvec::{order::Msb0, slice::BitSlice, vec::BitVec};
use impl_tools::autoimpl;

/// Adapter that maps an error using given closure
#[autoimpl(Deref using self.inner)]
#[derive(Debug, Clone)]
pub struct MapErr<T, F> {
    pub(crate) inner: T,
    pub(crate) f: F,
}

impl<'de, R, F, E> BitReader<'de> for MapErr<R, F>
where
    R: BitReader<'de>,
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
    fn read_bits(&mut self, n: usize) -> Result<Cow<'de, BitSlice<u8, Msb0>>, Self::Error> {
        self.inner.read_bits(n).map_err(&mut self.f)
    }

    #[inline]
    fn skip(&mut self, n: usize) -> Result<usize, Self::Error> {
        self.inner.skip(n).map_err(&mut self.f)
    }
}

/// Adapter returned by [`.limit()`](crate::ser::BitWriterExt::limit)
#[autoimpl(Deref using self.inner)]
#[derive(Debug, Clone)]
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
        if self.capacity_left() < n {
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
    fn capacity_left(&self) -> usize {
        (self.limit - self.bit_count()).min(self.inner.capacity_left())
    }

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

/// `tee`-like adapter for mirroring data read/written
#[autoimpl(Deref using self.inner)]
#[derive(Debug, Clone)]
pub struct Tee<T, W> {
    inner: T,
    writer: W,
}

impl<T, W> Tee<T, W> {
    pub(crate) fn new(inner: T, writer: W) -> Self {
        Self { inner, writer }
    }

    #[inline]
    pub fn into_inner(self) -> T {
        self.inner
    }

    #[inline]
    pub fn into_writer(self) -> W {
        self.writer
    }
}

impl<'de, R, W> BitReader<'de> for Tee<R, W>
where
    R: BitReader<'de>,
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

    #[inline]
    fn read_bits(&mut self, n: usize) -> Result<Cow<'de, BitSlice<u8, Msb0>>, Self::Error> {
        let v = self.inner.read_bits(n)?;
        self.writer
            .write_bitslice(&v)
            .map_err(|err| <R::Error>::custom(err).context("writer"))?;
        Ok(v)
    }
}

impl<T, W> BitWriter for Tee<T, W>
where
    T: BitWriter,
    W: BitWriter,
{
    type Error = T::Error;

    #[inline]
    fn capacity_left(&self) -> usize {
        self.inner.capacity_left().min(self.writer.capacity_left())
    }

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

#[autoimpl(Deref using self.0)]
#[derive(Debug, Clone)]
pub struct Checkpoint<R>(Tee<R, BitVec<u8, Msb0>>);

impl<R> Checkpoint<R> {
    #[inline]
    pub(crate) fn new(r: R) -> Self {
        Self(Tee::new(r, BitVec::new()))
    }

    #[inline]
    pub fn restore<'de>(self) -> Join<impl BitReader<'de>, R>
    where
        R: BitReader<'de>,
    {
        Owned::new(self.0.writer).join(self.0.inner)
    }
}

impl<'de, R> BitReader<'de> for Checkpoint<R>
where
    R: BitReader<'de>,
{
    type Error = <Tee<R, BitVec<u8, Msb0>> as BitReader<'de>>::Error;

    #[inline]
    fn bits_left(&self) -> usize {
        self.0.bits_left()
    }

    #[inline]
    fn read_bit(&mut self) -> Result<Option<bool>, Self::Error> {
        self.0.read_bit()
    }

    #[inline]
    fn read_bits_into(&mut self, dst: &mut BitSlice<u8, Msb0>) -> Result<usize, Self::Error> {
        self.0.read_bits_into(dst)
    }

    #[inline]
    fn read_bits(&mut self, n: usize) -> Result<Cow<'de, BitSlice<u8, Msb0>>, Self::Error> {
        self.0.read_bits(n)
    }

    #[inline]
    fn skip(&mut self, n: usize) -> Result<usize, Self::Error> {
        self.0.skip(n)
    }
}

/// Adapter for counting the number of bits read/written.
#[autoimpl(Deref using self.inner)]
#[derive(Debug, Clone)]
pub struct BitCounter<T> {
    pub(crate) inner: T,
    pub(crate) counter: usize,
}

impl<T> BitCounter<T> {
    #[inline]
    pub const fn new(inner: T) -> Self {
        Self { inner, counter: 0 }
    }

    /// Return total number of recorded bits
    #[inline]
    pub const fn bit_count(&self) -> usize {
        self.counter
    }

    #[inline]
    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<'de, R> BitReader<'de> for BitCounter<R>
where
    R: BitReader<'de>,
{
    type Error = R::Error;

    #[inline]
    fn bits_left(&self) -> usize {
        self.inner.bits_left()
    }

    #[inline]
    fn read_bit(&mut self) -> Result<Option<bool>, Self::Error> {
        let bit = self.inner.read_bit()?;
        if bit.is_some() {
            self.counter += 1;
        }
        Ok(bit)
    }

    #[inline]
    fn read_bits_into(&mut self, dst: &mut BitSlice<u8, Msb0>) -> Result<usize, Self::Error> {
        let n = self.inner.read_bits_into(dst)?;
        self.counter += n;
        Ok(n)
    }

    #[inline]
    fn read_bits(&mut self, n: usize) -> Result<Cow<'de, BitSlice<u8, Msb0>>, Self::Error> {
        let v = self.inner.read_bits(n)?;
        self.counter += v.len();
        Ok(v)
    }

    #[inline]
    fn skip(&mut self, n: usize) -> Result<usize, Self::Error> {
        let s = self.inner.skip(n)?;
        self.counter += s;
        Ok(s)
    }
}

impl<W> BitWriter for BitCounter<W>
where
    W: BitWriter,
{
    type Error = W::Error;

    #[inline]
    fn capacity_left(&self) -> usize {
        self.inner.capacity_left()
    }

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

#[derive(Debug, Clone)]
pub struct Join<T1, T2>(T1, T2);

impl<T1, T2> Join<T1, T2> {
    #[inline]
    pub(crate) fn new(a: T1, b: T2) -> Self {
        Self(a, b)
    }

    #[inline]
    pub fn into_inner(self) -> (T1, T2) {
        (self.0, self.1)
    }
}

impl<'de, R1, R2> BitReader<'de> for Join<R1, R2>
where
    R1: BitReader<'de>,
    R2: BitReader<'de>,
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
    fn read_bits(&mut self, n: usize) -> Result<Cow<'de, BitSlice<u8, Msb0>>, Self::Error> {
        let v0 = self.0.read_bits(n)?;
        let v1 = self.1.read_bits(n - v0.len()).map_err(Error::custom)?;
        Ok(if v1.is_empty() {
            v0
        } else if v0.is_empty() {
            v1
        } else {
            let mut v = v0.into_owned();
            v.extend_from_bitslice(&v1);
            Cow::Owned(v)
        })
    }

    #[inline]
    fn skip(&mut self, n: usize) -> Result<usize, Self::Error> {
        let skipped = self.0.skip(n)?;
        Ok(skipped + self.1.skip(n - skipped).map_err(Error::custom)?)
    }
}

#[derive(Debug, Clone)]
pub struct Owned {
    inner: BitCounter<BitVec<u8, Msb0>>,
    rest: *const BitSlice<u8, Msb0>,
}

impl Owned {
    pub fn new(bits: BitVec<u8, Msb0>) -> Self {
        Self {
            rest: bits.as_bitslice(),
            inner: BitCounter::new(bits),
        }
    }

    #[inline]
    pub fn rest<'a>(&self) -> &'a BitSlice<u8, Msb0> {
        // TODO
        unsafe { self.rest.as_ref().unwrap_unchecked() }
    }

    fn advance(&mut self, n: usize) {
        self.inner.counter += n;
    }
}

impl<'de> BitReader<'de> for Owned {
    type Error = <&'de BitSlice<u8, Msb0> as BitReader<'de>>::Error;

    #[inline]
    fn bits_left(&self) -> usize {
        self.rest().len()
    }

    #[inline]
    fn read_bit(&mut self) -> Result<Option<bool>, Self::Error> {
        let bit = self.rest().read_bit()?;
        if bit.is_some() {
            self.advance(1);
        }
        Ok(bit)
    }

    #[inline]
    fn read_bits_into(&mut self, dst: &mut BitSlice<u8, Msb0>) -> Result<usize, Self::Error> {
        let n = self.rest().read_bits_into(dst)?;
        self.advance(n);
        Ok(n)
    }

    #[inline]
    fn read_bits(&mut self, n: usize) -> Result<Cow<'de, BitSlice<u8, Msb0>>, Self::Error> {
        let v = self.rest().read_bits(n)?;
        self.advance(v.len());
        Ok(v)
    }

    #[inline]
    fn skip(&mut self, n: usize) -> Result<usize, Self::Error> {
        let n = self.rest().skip(n)?;
        self.advance(n);
        Ok(n)
    }
}
