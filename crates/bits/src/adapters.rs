//! Adapters for [`BitReader`](crate::de::BitReader) / [`BitWriter`](crate::ser::BitWriter)
use std::mem;

use bitvec::{
    array::BitArray, bitarr, bits, index::BitIdx, mem::bits_of, order::Msb0, slice::BitSlice,
    view::BitView,
};
use impl_tools::autoimpl;

/// Adapter that maps an error using given closure
#[autoimpl(Deref using self.inner)]
pub struct MapErr<T, F> {
    pub(crate) inner: T,
    pub(crate) f: F,
}

/// `tee`-like adapter for mirroring data read/written
#[autoimpl(Deref using self.inner)]
pub struct Tee<T, W> {
    pub(crate) inner: T,
    pub(crate) writer: W,
}

impl<T, W> Tee<T, W> {
    #[inline]
    pub fn into_inner(self) -> T {
        self.inner
    }

    #[inline]
    pub fn into_writer(self) -> W {
        self.writer
    }
}

/// Adapter for counting the number of bits read/written.
#[autoimpl(Deref using self.inner)]
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

pub struct Join<T1, T2>(pub T1, pub T2);

// TODO
pub struct Iter<T>(T);

// TODO: maybe remove BUF_LEN?
#[derive(Debug, Clone, Default)]
pub struct Io<T, const BUF_LEN: usize = 1> {
    pub(crate) io: T,
    // TODO: docs
    // 1|0 0.0 0 0 1 1
    // 0 0 1|0 0 0 1 1
    pub(crate) buf: BitArray<[u8; BUF_LEN], Msb0>,
}

impl<T, const BUF_LEN: usize> Io<T, BUF_LEN> {
    const _CHECK_BUF_LEN: () = assert!(BUF_LEN > 0);

    #[inline]
    pub fn new(io: T) -> Self {
        let _ = Self::_CHECK_BUF_LEN;

        let mut v = Self {
            io,
            buf: BitArray::ZERO,
        };
        v.reset_buf();
        v
    }

    #[inline]
    pub(crate) fn buffer(&self) -> &BitSlice<u8, Msb0> {
        unsafe { self.buf.get_unchecked(self.buf.leading_zeros() + 1..) }
    }

    #[inline]
    pub(crate) fn buffer_full(&self) -> bool {
        self.buf.leading_zeros() == 0
    }

    #[inline]
    pub(crate) fn reset_buf(&mut self) -> [u8; BUF_LEN] {
        let prev = mem::replace(&mut self.buf, BitArray::ZERO);
        *unsafe { self.buf.last_mut().unwrap_unchecked() } = true;
        prev.into_inner()
    }

    #[must_use]
    pub(crate) fn buf_skip_at_most(&mut self, n: usize) -> usize {
        let old_stop = self.buf.leading_zeros();
        let new_stop = (old_stop + n).min(self.buf.len() - 1);
        unsafe {
            self.buf.set_unchecked(new_stop, true);
            self.buf.get_unchecked_mut(old_stop..new_stop)
        }
        .fill(false);
        new_stop - old_stop
    }

    /// Returns if flush is needed
    #[inline]
    pub(crate) fn buf_put(&mut self, bit: bool) -> Option<[u8; BUF_LEN]> {
        let flush = self.buffer_full();
        self.buf.shift_left(1);
        *unsafe { self.buf.last_mut().unwrap_unchecked() } = bit;
        flush.then(|| self.reset_buf())
    }

    #[inline]
    pub fn into_inner(self) -> Option<T> {
        self.buffer().is_empty().then_some(self.io)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        r#as::{NBits, Same},
        de::BitReaderExt,
        ser::BitWriterExt,
    };

    use super::*;

    #[test]
    fn io() {
        type T = (u64, bool);
        type As = (NBits<63>, Same);
        const VALUE: T = (0x7e40f1e8ceabc94d, true);

        let mut buf: Io<_, 2> = Io::new(Vec::<u8>::new());
        buf.pack_as::<_, As>(VALUE).unwrap();
        let buf = buf.into_inner().unwrap();

        let mut buf: Io<_, 4> = Io::new(buf.as_slice());
        let got: T = buf.unpack_as::<_, As>().unwrap();
        assert_eq!(got, VALUE);
    }
}
