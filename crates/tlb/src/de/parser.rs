use core::iter;
use std::sync::Arc;

use tlbits::ResultExt;

use crate::{
    bits::{
        bitvec::{order::Msb0, slice::BitSlice},
        de::BitReader,
    },
    Cell, Error,
};

use super::{
    args::{r#as::CellDeserializeAsWithArgs, CellDeserializeWithArgs},
    r#as::CellDeserializeAs,
    CellDeserialize,
};

pub type CellParserError<'de> = <CellParser<'de> as BitReader>::Error;

pub struct CellParser<'de> {
    pub(super) data: &'de BitSlice<u8, Msb0>,
    pub(super) references: &'de [Arc<Cell>],
}

impl<'de> CellParser<'de> {
    #[inline]
    pub(crate) const fn new(data: &'de BitSlice<u8, Msb0>, references: &'de [Arc<Cell>]) -> Self {
        Self { data, references }
    }

    #[inline]
    pub fn parse<T>(&mut self) -> Result<T, CellParserError<'de>>
    where
        T: CellDeserialize<'de>,
    {
        T::parse(self)
    }

    #[inline]
    pub fn parse_iter<T>(&mut self) -> impl Iterator<Item = Result<T, CellParserError<'de>>> + '_
    where
        T: CellDeserialize<'de>,
    {
        iter::repeat_with(move || self.parse())
            .enumerate()
            .map(|(i, v)| v.with_context(|| format!("[{i}]")))
    }

    #[inline]
    pub fn parse_with<T>(&mut self, args: T::Args) -> Result<T, CellParserError<'de>>
    where
        T: CellDeserializeWithArgs<'de>,
    {
        T::parse_with(self, args)
    }

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

    #[inline]
    pub fn parse_as<T, As>(&mut self) -> Result<T, CellParserError<'de>>
    where
        As: CellDeserializeAs<'de, T> + ?Sized,
    {
        As::parse_as(self)
    }

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

    #[inline]
    pub fn parse_as_with<T, As>(&mut self, args: As::Args) -> Result<T, CellParserError<'de>>
    where
        As: CellDeserializeAsWithArgs<'de, T> + ?Sized,
    {
        As::parse_as_with(self, args)
    }

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
    pub fn is_empty(&self) -> bool {
        self.data.is_empty() && self.references.is_empty()
    }

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
    fn read_bit(&mut self) -> Result<bool, Self::Error> {
        self.data.read_bit()
    }

    #[inline]
    fn read_bits_into(&mut self, dst: &mut BitSlice<u8, Msb0>) -> Result<(), Self::Error> {
        self.data.read_bits_into(dst)
    }

    #[inline]
    fn skip(&mut self, n: usize) -> Result<(), Self::Error> {
        self.data.skip(n)
    }
}
