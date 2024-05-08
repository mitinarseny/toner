use bitvec::{order::Msb0, slice::BitSlice};

use crate::{BitReader, Error};

pub struct MapErr<T, F> {
    pub(crate) inner: T,
    pub(crate) f: F,
}

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
