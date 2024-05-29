use impl_tools::autoimpl;

#[autoimpl(Deref using self.inner)]
pub struct MapErr<T, F> {
    pub(crate) inner: T,
    pub(crate) f: F,
}

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

    #[inline]
    pub const fn counter(&self) -> usize {
        self.counter
    }

    #[inline]
    pub fn into_inner(self) -> T {
        self.inner
    }
}
