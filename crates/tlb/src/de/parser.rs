use core::{iter, mem};
use std::{borrow::Cow, sync::Arc};

use tlbits::Context;

use crate::{
    Cell, Error,
    bits::{
        bitvec::{order::Msb0, slice::BitSlice},
        de::BitReader,
    },
};

use super::{CellDeserialize, CellDeserializeAs};

/// [`Error`] for [`CellParser`]
pub type CellParserError<'de> = <CellParser<'de> as BitReader<'de>>::Error;

/// Cell parser created with [`Cell::parser()`].
#[derive(Clone)]
pub struct CellParser<'de> {
    pub(super) data: &'de BitSlice<u8, Msb0>,
    pub(super) references: &'de [Arc<Cell>],
}

impl<'de> CellParser<'de> {
    #[inline]
    pub(crate) const fn new(data: &'de BitSlice<u8, Msb0>, references: &'de [Arc<Cell>]) -> Self {
        Self { data, references }
    }

    /// Parse the value with args using its [`CellDeserialize`]
    /// implementation.
    #[inline]
    pub fn parse<T>(&mut self, args: T::Args) -> Result<T, CellParserError<'de>>
    where
        T: CellDeserialize<'de>,
    {
        T::parse(self, args)
    }

    /// Return iterator that parses values with args using
    /// [`CellDeserialize`] implementation.
    #[inline]
    pub fn parse_iter<'a: 'de, T>(
        &mut self,
        args: T::Args,
    ) -> impl Iterator<Item = Result<T, CellParserError<'de>>> + '_
    where
        T: CellDeserialize<'de>,
        T::Args: Clone + 'a,
    {
        iter::repeat_with(move || self.parse(args.clone()))
            .enumerate()
            .map(|(i, v)| v.with_context(|| format!("[{i}]")))
    }

    /// Parse value with args using an adapter.  
    ///
    /// This approach is heavily inspired by
    /// [serde_with](https://docs.rs/serde_with/latest/serde_with).
    /// Please, read their docs for more usage examples.
    #[inline]
    pub fn parse_as<T, As>(&mut self, args: As::Args) -> Result<T, CellParserError<'de>>
    where
        As: CellDeserializeAs<'de, T> + ?Sized,
    {
        As::parse_as(self, args)
    }

    /// Returns iterator that parses values with args using an adapter.  
    ///
    /// This approach is heavily inspired by
    /// [serde_with](https://docs.rs/serde_with/latest/serde_with).
    /// Please, read their docs for more usage examples.
    #[inline]
    pub fn parse_iter_as<'a: 'de, T, As>(
        &mut self,
        args: As::Args,
    ) -> impl Iterator<Item = Result<T, CellParserError<'de>>> + '_
    where
        As: CellDeserializeAs<'de, T> + ?Sized,
        As::Args: Clone + 'a,
    {
        iter::repeat_with(move || self.parse_as::<_, As>(args.clone()))
            .enumerate()
            .map(|(i, v)| v.with_context(|| format!("[{i}]")))
    }

    #[inline]
    fn pop_reference(&mut self) -> Result<&'de Arc<Cell>, CellParserError<'de>> {
        let (first, rest) = self
            .references
            .split_first()
            .ok_or_else(|| Error::custom("no more references left"))?;
        self.references = rest;
        Ok(first)
    }

    #[inline]
    pub(crate) fn parse_reference_as<T, As>(
        &mut self,
        args: As::Args,
    ) -> Result<T, CellParserError<'de>>
    where
        As: CellDeserializeAs<'de, T> + ?Sized,
    {
        self.pop_reference()?.parse_fully_as::<T, As>(args)
    }

    #[inline]
    pub fn bits_left(&self) -> usize {
        self.data.len()
    }

    #[inline]
    pub fn no_bits_left(&self) -> bool {
        self.bits_left() == 0
    }

    #[inline]
    pub const fn references_left(&self) -> usize {
        self.references.len()
    }

    #[inline]
    pub const fn no_references_left(&self) -> bool {
        self.references_left() == 0
    }

    /// Returns whether this parser has no more data and references.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.no_bits_left() && self.no_references_left()
    }

    /// Returns an error if this parser has more data or references.
    #[inline]
    pub fn ensure_empty(&self) -> Result<(), CellParserError<'de>> {
        if !self.is_empty() {
            return Err(Error::custom(format!(
                "more data left: {} bits, {} references",
                self.data.len(),
                self.references.len(),
            )));
        }
        Ok(())
    }
}

impl<'de> BitReader<'de> for CellParser<'de> {
    type Error = <&'de BitSlice<u8, Msb0> as BitReader<'de>>::Error;

    #[inline]
    fn bits_left(&self) -> usize {
        self.data.len()
    }

    #[inline]
    fn read_bit(&mut self) -> Result<Option<bool>, Self::Error> {
        self.data.read_bit()
    }

    #[inline]
    fn read_bits_into(&mut self, dst: &mut BitSlice<u8, Msb0>) -> Result<usize, Self::Error> {
        self.data.read_bits_into(dst)
    }

    #[inline]
    fn read_bits(&mut self, n: usize) -> Result<Cow<'de, BitSlice<u8, Msb0>>, Self::Error> {
        self.data.read_bits(n)
    }

    #[inline]
    fn skip(&mut self, n: usize) -> Result<usize, Self::Error> {
        self.data.skip(n)
    }
}

impl<'de> CellDeserialize<'de> for CellParser<'de> {
    type Args = ();

    #[inline]
    fn parse(parser: &mut CellParser<'de>, _: Self::Args) -> Result<Self, CellParserError<'de>> {
        Ok(Self {
            data: mem::take(&mut parser.data),
            references: mem::take(&mut parser.references),
        })
    }
}
