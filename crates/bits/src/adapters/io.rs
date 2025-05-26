use std::{
    fmt::Display,
    io::{self, Read, Write},
    mem,
};

use bitvec::{array::BitArray, mem::bits_of, order::Msb0, slice::BitSlice};

use crate::{Error, de::BitReader, ser::BitWriter};

type Buffer = BitArray<[u8; 1], Msb0>;

/// Binary adaptor for [`io::Read`] and [`io::Write`] with bit-level granularity
/// ```rust
/// # use std::io;
/// #
/// # use tlbits::{
/// #     adapters::Io,
/// #     r#as::NBits,
/// #     de::BitReaderExt,
/// #     ser::BitWriterExt
/// # };
/// # fn main() -> Result<(), io::Error> {
/// // pack
/// let mut writer = Io::new(Vec::<u8>::new());
/// writer.pack_as::<u8, NBits<7>>(123)?
///     .pack(true)?;
/// let buf = writer.into_inner().unwrap();
///
/// // unpack
/// let mut reader = Io::new(buf.as_slice());
/// let value1 = reader.unpack_as::<u8, NBits<7>>()?;
/// let value2 = reader.unpack::<bool>()?;
///
/// # assert!(reader.buffered().is_empty());
/// # assert_eq!(value1, 123);
/// # assert_eq!(value2, true);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default)]
pub struct Io<T> {
    /// Buffer for not-yet-consumed or not-yet-flushed data.
    /// Bits are stored in [`Msb0`] order and populated from the right.
    /// Left-most set bit denotes the end of buffer.
    ///
    /// For example, `0b00011` is stored as `0 0 1|0 0 0 1 1`.
    buf: Buffer,
    io: T,
}

impl<T> Io<T> {
    const BUF_LEN: usize = bits_of::<Buffer>();

    #[inline]
    pub fn new(io: T) -> Self {
        let mut s = Self {
            buf: Buffer::ZERO,
            io,
        };
        let _ = s.reset_buf();
        s
    }

    #[inline]
    pub fn buffered(&self) -> &BitSlice<u8, Msb0> {
        unsafe { self.buf.get_unchecked(self.buf.leading_zeros() + 1..) }
    }

    #[inline]
    pub(crate) fn buffer_capacity_left(&self) -> usize {
        self.buf.leading_zeros() + 1
    }

    #[must_use]
    #[inline]
    pub(crate) fn reset_buf(&mut self) -> [u8; 1] {
        let prev = mem::replace(&mut self.buf, Buffer::ZERO);
        unsafe {
            self.buf.set_unchecked(Self::BUF_LEN - 1, true);
        }
        prev.into_inner()
    }

    #[must_use]
    #[inline]
    pub(crate) fn buf_skip_at_most(&mut self, n: usize) -> usize {
        let old_stop = self.buf.leading_zeros();
        let new_stop = (old_stop + n).min(Self::BUF_LEN - 1);
        unsafe {
            self.buf.set_unchecked(new_stop, true);
            self.buf.get_unchecked_mut(old_stop..new_stop)
        }
        .fill(false);
        new_stop - old_stop
    }

    #[must_use]
    #[inline]
    pub fn into_inner(self) -> Option<T> {
        self.buffered()
            .is_empty()
            .then_some(self.into_inner_unchecked())
    }

    #[inline]
    pub fn into_inner_unchecked(self) -> T {
        self.io
    }
}

impl<W> Io<W>
where
    W: Write,
{
    pub fn fill_up_buffer_and_flush(
        &mut self,
        fill_bit: bool,
    ) -> Result<usize, <Self as BitWriter>::Error> {
        Ok(if !self.buffered().is_empty() {
            let n = self.buffer_capacity_left();
            self.repeat_bit(n, fill_bit)?;
            n
        } else {
            0
        })
    }
}

impl<R> BitReader for Io<R>
where
    R: Read,
{
    type Error = io::Error;

    #[inline]
    fn bits_left(&self) -> usize {
        usize::MAX
    }

    fn read_bit(&mut self) -> Result<Option<bool>, Self::Error> {
        let stop = if self.buffered().is_empty() {
            match self.io.read(self.buf.as_raw_mut_slice())? {
                0 => return Ok(None),
                1 => 0,
                _ => unreachable!(),
            }
        } else {
            let old_stop = self.buf.leading_zeros();
            unsafe { self.buf.set_unchecked(old_stop, false) };
            old_stop + 1
        };
        Ok(Some(unsafe {
            // put stop-bit
            self.buf.replace_unchecked(stop, true)
        }))
    }

    fn read_bits_into(&mut self, mut rest: &mut BitSlice<u8, Msb0>) -> Result<usize, Self::Error> {
        let init_len = rest.len();
        while !rest.is_empty() {
            let buf = self.buffered();
            if !buf.is_empty() {
                let buf_n = buf.len().min(rest.len());
                rest = unsafe {
                    rest.get_unchecked_mut(..buf_n)
                        .copy_from_bitslice(buf.get_unchecked(..buf_n));
                    rest.get_unchecked_mut(buf_n..)
                };
                let _ = self.buf_skip_at_most(buf_n);
                continue;
            }
            // buf is empty here

            if let Some((head, body, _tail)) = rest
                .domain_mut()
                .region()
                .filter(|(_head, body, _tail)| !body.is_empty())
            {
                let bytes = self.io.read(body)?;
                if bytes == 0 {
                    break;
                }
                let shift_to_head = head.map_or(0, |p| p.into_bitslice().len());
                rest.shift_left(shift_to_head);
                rest = &mut rest[bytes * bits_of::<u8>()..];
                continue;
            }

            // this will populate the buffer
            let Some(bit) = self.read_bit()? else {
                break;
            };
            rest = unsafe {
                *rest.get_unchecked_mut(0) = bit;
                rest.get_unchecked_mut(1..)
            };
        }
        Ok(init_len - rest.len())
    }

    fn skip(&mut self, n: usize) -> Result<usize, Self::Error> {
        let mut rest = n;
        rest -= self.buf_skip_at_most(n);
        rest -= io::copy(
            &mut self.io.by_ref().take((rest / bits_of::<u8>()) as u64),
            &mut io::sink(),
        )? as usize
            * bits_of::<u8>();
        while rest > 0 && self.read_bit()?.is_some() {
            rest -= 1;
        }
        Ok(n - rest)
    }
}

impl<W> BitWriter for Io<W>
where
    W: Write,
{
    type Error = io::Error;

    #[inline]
    fn capacity_left(&self) -> usize {
        usize::MAX
    }

    #[inline]
    fn write_bit(&mut self, bit: bool) -> Result<(), Self::Error> {
        let flush = self.buffer_capacity_left() == 1;
        self.buf.shift_left(1);
        unsafe { self.buf.set_unchecked(Self::BUF_LEN - 1, bit) };
        if flush {
            let buf = self.reset_buf();
            self.io.write_all(&buf)?;
        }
        Ok(())
    }

    fn write_bitslice(&mut self, mut bits: &BitSlice<u8, Msb0>) -> Result<(), Self::Error> {
        while !bits.is_empty() {
            if self.buffered().is_empty() {
                if let Some(body) = bits
                    .domain()
                    .region()
                    .and_then(|(head, body, _tail)| head.is_none().then_some(body))
                {
                    self.io.write_all(body)?;
                    bits = unsafe { bits.get_unchecked(body.len() * bits_of::<u8>()..) };
                    continue;
                }
            }

            let buf_cap_left = self.buffer_capacity_left();
            let n = bits.len().min(buf_cap_left);
            let flush = n == buf_cap_left;
            self.buf.shift_left(n);
            bits = unsafe {
                self.buf
                    .get_unchecked_mut(Self::BUF_LEN - n..)
                    .copy_from_bitslice(bits.get_unchecked(..n));
                bits.get_unchecked(n..)
            };
            if flush {
                let buf = self.reset_buf();
                self.io.write_all(&buf)?;
            }
        }
        Ok(())
    }
}

impl Error for io::Error {
    #[inline]
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Self::other(msg.to_string())
    }

    #[inline]
    fn context<C>(self, context: C) -> Self
    where
        C: Display,
    {
        Self::new(self.kind(), format!("{context}: {self}"))
    }
}
