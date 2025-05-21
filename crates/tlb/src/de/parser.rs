use core::{iter, mem};
use std::sync::Arc;

use tlbits::Context;

use crate::{
    Cell, Error,
    bits::{
        bitvec::{order::Msb0, slice::BitSlice},
        de::BitReader,
    },
};

use super::{
    CellDeserialize,
    args::{CellDeserializeWithArgs, r#as::CellDeserializeAsWithArgs},
    r#as::CellDeserializeAs,
};

/// [`Error`] for [`CellParser`]
pub type CellParserError<'de> = <CellParser<'de> as BitReader>::Error;

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

    /// Parse the value using its [`CellDeserialize`] implementation
    #[inline]
    pub fn parse<T>(&mut self) -> Result<T, CellParserError<'de>>
    where
        T: CellDeserialize<'de>,
    {
        T::parse(self)
    }

    /// Return iterator that parses values using [`CellDeserialize`]
    /// implementation.
    #[inline]
    pub fn parse_iter<T>(&mut self) -> impl Iterator<Item = Result<T, CellParserError<'de>>> + '_
    where
        T: CellDeserialize<'de>,
    {
        iter::repeat_with(move || self.parse())
            .enumerate()
            .map(|(i, v)| v.with_context(|| format!("[{i}]")))
    }

    /// Parse the value with args using its [`CellDeserializeWithArgs`]
    /// implementation.
    #[inline]
    pub fn parse_with<T>(&mut self, args: T::Args) -> Result<T, CellParserError<'de>>
    where
        T: CellDeserializeWithArgs<'de>,
    {
        T::parse_with(self, args)
    }

    /// Return iterator that parses values with args using
    /// [`CellDeserializeWithArgs`] implementation.
    #[inline]
    pub fn parse_iter_with<'a: 'de, T>(
        &mut self,
        args: T::Args,
    ) -> impl Iterator<Item = Result<T, CellParserError<'de>>> + '_
    where
        T: CellDeserializeWithArgs<'de>,
        T::Args: Clone + 'a,
    {
        iter::repeat_with(move || self.parse_with(args.clone()))
            .enumerate()
            .map(|(i, v)| v.with_context(|| format!("[{i}]")))
    }

    /// Parse the value using an adapter.  
    /// See [`as`](crate::as) module-level documentation for more.
    #[inline]
    pub fn parse_as<T, As>(&mut self) -> Result<T, CellParserError<'de>>
    where
        As: CellDeserializeAs<'de, T> + ?Sized,
    {
        As::parse_as(self)
    }

    /// Returns iterator that parses values using an adapter.  
    /// See [`as`](crate::as) module-level documentation for more.
    #[inline]
    pub fn parse_iter_as<T, As>(
        &mut self,
    ) -> impl Iterator<Item = Result<T, CellParserError<'de>>> + '_
    where
        As: CellDeserializeAs<'de, T> + ?Sized,
    {
        iter::repeat_with(move || self.parse_as::<_, As>())
            .enumerate()
            .map(|(i, v)| v.with_context(|| format!("[{i}]")))
    }

    /// Parse value with args using an adapter.  
    /// See [`as`](crate::as) module-level documentation for more.
    #[inline]
    pub fn parse_as_with<T, As>(&mut self, args: As::Args) -> Result<T, CellParserError<'de>>
    where
        As: CellDeserializeAsWithArgs<'de, T> + ?Sized,
    {
        As::parse_as_with(self, args)
    }

    /// Returns iterator that parses values with args using an adapter.  
    /// See [`as`](crate::as) module-level documentation for more.
    #[inline]
    pub fn parse_iter_as_with<'a: 'de, T, As>(
        &mut self,
        args: As::Args,
    ) -> impl Iterator<Item = Result<T, CellParserError<'de>>> + '_
    where
        As: CellDeserializeAsWithArgs<'de, T> + ?Sized,
        As::Args: Clone + 'a,
    {
        iter::repeat_with(move || self.parse_as_with::<_, As>(args.clone()))
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
    pub(crate) fn parse_reference_as<T, As>(&mut self) -> Result<T, CellParserError<'de>>
    where
        As: CellDeserializeAs<'de, T> + ?Sized,
    {
        self.pop_reference()?.parse_fully_as::<T, As>()
    }

    #[inline]
    pub(crate) fn parse_reference_as_with<T, As>(
        &mut self,
        args: As::Args,
    ) -> Result<T, CellParserError<'de>>
    where
        As: CellDeserializeAsWithArgs<'de, T> + ?Sized,
    {
        self.pop_reference()?.parse_fully_as_with::<T, As>(args)
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

impl<'de> BitReader for CellParser<'de> {
    type Error = <&'de BitSlice<u8, Msb0> as BitReader>::Error;

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
    fn skip(&mut self, n: usize) -> Result<usize, Self::Error> {
        self.data.skip(n)
    }
}

impl<'de> CellDeserialize<'de> for CellParser<'de> {
    #[inline]
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, CellParserError<'de>> {
        Ok(Self {
            data: mem::take(&mut parser.data),
            references: mem::take(&mut parser.references),
        })
    }
}
