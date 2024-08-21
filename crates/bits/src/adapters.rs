//! Adapters for [`BitReader`](crate::de::BitReader) / [`BitWriter`](crate::ser::BitWriter)
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
