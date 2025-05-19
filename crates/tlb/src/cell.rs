use core::{
    fmt::{self, Debug},
    hash::Hash,
    ops::Deref,
};
use std::sync::Arc;

use bitvec::{order::Msb0, vec::BitVec};
use sha2::{Digest, Sha256};

use crate::{
    de::{
        CellDeserialize, CellParser, CellParserError,
        args::{CellDeserializeWithArgs, r#as::CellDeserializeAsWithArgs},
        r#as::CellDeserializeAs,
    },
    ser::CellBuilder,
};

/// A [Cell](https://docs.ton.org/develop/data-formats/cell-boc#cell).  
#[derive(Clone, Default, PartialEq, Eq, Hash)]
pub struct Cell {
    pub data: BitVec<u8, Msb0>,
    pub references: Vec<Arc<Self>>,
}

impl Cell {
    /// Create new [`CellBuilder`]
    #[inline]
    #[must_use]
    pub const fn builder() -> CellBuilder {
        CellBuilder::new()
    }

    /// Create empty cell
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            data: BitVec::EMPTY,
            references: Vec::new(),
        }
    }

    /// Return [`CellParser`] for this cell
    #[inline]
    #[must_use]
    pub fn parser(&self) -> CellParser<'_> {
        CellParser::new(&self.data, &self.references)
    }

    /// Shortcut for [`.parser()`](Cell::parser)[`.parse()`](CellParser::parse)[`.ensure_empty()`](CellParser::ensure_empty).
    #[inline]
    pub fn parse_fully<'de, T>(&'de self) -> Result<T, CellParserError<'de>>
    where
        T: CellDeserialize<'de>,
    {
        let mut parser = self.parser();
        let v = parser.parse()?;
        parser.ensure_empty()?;
        Ok(v)
    }

    /// Shortcut for [`.parser()`](Cell::parser)[`.parse_with()`](CellParser::parse_with)[`.ensure_empty()`](CellParser::ensure_empty).
    #[inline]
    pub fn parse_fully_with<'de, T>(&'de self, args: T::Args) -> Result<T, CellParserError<'de>>
    where
        T: CellDeserializeWithArgs<'de>,
    {
        let mut parser = self.parser();
        let v = parser.parse_with(args)?;
        parser.ensure_empty()?;
        Ok(v)
    }

    /// Shortcut for [`.parser()`](Cell::parser)[`.parse_as()`](CellParser::parse_as)[`.ensure_empty()`](CellParser::ensure_empty).
    #[inline]
    pub fn parse_fully_as<'de, T, As>(&'de self) -> Result<T, CellParserError<'de>>
    where
        As: CellDeserializeAs<'de, T> + ?Sized,
    {
        let mut parser = self.parser();
        let v = parser.parse_as::<T, As>()?;
        parser.ensure_empty()?;
        Ok(v)
    }

    /// Shortcut for [`.parser()`](Cell::parser)[`.parse_as_with()`](CellParser::parse_as_with)[`.ensure_empty()`](CellParser::ensure_empty).
    #[inline]
    pub fn parse_fully_as_with<'de, T, As>(
        &'de self,
        args: As::Args,
    ) -> Result<T, CellParserError<'de>>
    where
        As: CellDeserializeAsWithArgs<'de, T> + ?Sized,
    {
        let mut parser = self.parser();
        let v = parser.parse_as_with::<T, As>(args)?;
        parser.ensure_empty()?;
        Ok(v)
    }

    /// Returns whether this cell has no data and zero references.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty() && self.references.is_empty()
    }

    #[inline]
    fn data_bytes(&self) -> (usize, &[u8]) {
        (self.data.len(), self.data.as_raw_slice())
    }

    /// See [Cell level](https://docs.ton.org/develop/data-formats/cell-boc#cell-level)
    #[inline]
    pub fn level(&self) -> u8 {
        self.references
            .iter()
            .map(Deref::deref)
            .map(Cell::level)
            .max()
            .unwrap_or(0)
    }

    /// See [Cell serialization](https://docs.ton.org/develop/data-formats/cell-boc#cell-serialization)
    #[inline]
    fn refs_descriptor(&self) -> u8 {
        // TODO: exotic cells
        self.references.len() as u8 | (self.level() << 5)
    }

    /// See [Cell serialization](https://docs.ton.org/develop/data-formats/cell-boc#cell-serialization)
    #[inline]
    fn bits_descriptor(&self) -> u8 {
        let b = self.data.len();
        (b / 8) as u8 + b.div_ceil(8) as u8
    }

    #[inline]
    fn max_depth(&self) -> u16 {
        self.references
            .iter()
            .map(Deref::deref)
            .map(Cell::max_depth)
            .max()
            .map(|d| d + 1)
            .unwrap_or(0)
    }

    /// [Standard Cell representation hash](https://docs.ton.org/develop/data-formats/cell-boc#standard-cell-representation-hash-calculation)
    pub fn repr(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(self.refs_descriptor());
        buf.push(self.bits_descriptor());

        let rest_bits = self.data.len() % 8;

        if rest_bits == 0 {
            buf.extend(self.data.as_raw_slice());
        } else {
            let (last, data) = self.data.as_raw_slice().split_last().unwrap();
            buf.extend(data);
            let mut last = last & (!0u8 << (8 - rest_bits)); // clear the rest
            // let mut last = last;
            last |= 1 << (8 - rest_bits - 1); // put stop-bit
            buf.push(last)
        }

        // refs depth
        buf.extend(
            self.references
                .iter()
                .flat_map(|r| r.max_depth().to_be_bytes()),
        );

        // refs hashes
        buf.extend(
            self.references
                .iter()
                .map(Deref::deref)
                .flat_map(Cell::hash),
        );

        buf
    }

    /// Calculates [standard Cell representation hash](https://docs.ton.org/develop/data-formats/cell-boc#cell-hash)
    #[inline]
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(self.repr());
        hasher.finalize().into()
    }
}

impl Debug for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "{}[0b", self.data.len())?;
            for bit in &self.data {
                write!(f, "{}", if *bit { '1' } else { '0' })?;
            }
            write!(f, "]")?;
        } else {
            let (bits_len, data) = self.data_bytes();
            write!(f, "{}[0x{}]", bits_len, hex::encode_upper(data))?;
        }
        if self.references.is_empty() {
            return Ok(());
        }
        write!(f, " -> ")?;
        f.debug_set().entries(&self.references).finish()
    }
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use crate::{
        r#as::{Data, Ref},
        bits::{r#as::NBits, ser::BitWriterExt},
        ser::{CellSerializeExt, r#as::CellSerializeWrapAsExt},
        tests::assert_store_parse_as_eq,
    };

    use super::*;

    #[test]
    fn zero_depth() {
        assert_eq!(().to_cell().unwrap().max_depth(), 0)
    }

    #[test]
    fn max_depth() {
        let cell = (
            ().wrap_as::<Ref>(),
            (().wrap_as::<Ref>(), ().wrap_as::<Ref<Ref>>())
                .wrap_as::<Ref>()
                .wrap_as::<Ref>(),
            ((), ()),
        )
            .to_cell()
            .unwrap();
        assert_eq!(cell.max_depth(), 4)
    }

    #[test]
    fn cell_serde() {
        assert_store_parse_as_eq::<
            _,
            (
                Data<NBits<1>>,
                Ref<Data<NBits<24>>>,
                Ref<(Data<NBits<7>>, Ref<Data<NBits<24>>>)>,
            ),
        >((0b1, 0x0AAAAA, (0x7F, 0x0AAAAA)));
    }

    #[test]
    fn hash_no_refs() {
        let mut builder = Cell::builder();
        builder.pack_as::<_, NBits<32>>(0x0000000F).unwrap();
        let cell = builder.into_cell();

        assert_eq!(
            cell.hash(),
            hex!("57b520dbcb9d135863fc33963cde9f6db2ded1430d88056810a2c9434a3860f9")
        );
    }

    #[test]
    fn hash_with_refs() {
        let mut builder = Cell::builder();
        builder
            .store_as::<_, Data<NBits<24>>>(0x00000B)
            .unwrap()
            .store_reference_as::<_, Data>(0x0000000F_u32)
            .unwrap()
            .store_reference_as::<_, Data>(0x0000000F_u32)
            .unwrap();
        let cell = builder.into_cell();

        assert_eq!(
            cell.hash(),
            hex!("f345277cc6cfa747f001367e1e873dcfa8a936b8492431248b7a3eeafa8030e7")
        );
    }
}
