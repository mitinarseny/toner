mod boc_iter;
mod hasher;
mod iter;
mod kind;
mod level_mask;

pub use boc_iter::BagOfCellsIter;

use core::{
    fmt::{self, Debug},
    hash::Hash,
};
use std::sync::Arc;

use bitvec::{order::Msb0, vec::BitVec};
use digest::{Digest, Output};

use crate::{
    cell::{hasher::CellHasher, iter::CellIter, kind::ExoticCellKind, level_mask::LevelMask},
    de::{CellDeserialize, CellDeserializeAs, CellParser, CellParserError},
    ser::CellBuilder,
};

/// A [Cell](https://docs.ton.org/blockchain-basics/primitives/serialization/cells#cell).
#[derive(Clone, Default, PartialEq, Eq, Hash)]
pub struct Cell {
    pub is_exotic: bool,
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
            is_exotic: false,
            data: BitVec::EMPTY,
            references: Vec::new(),
        }
    }

    /// Return [`CellParser`] for this cell
    #[inline]
    #[must_use]
    pub fn parser(&self) -> CellParser<'_> {
        CellParser::new(self.is_exotic, &self.data, &self.references)
    }

    #[allow(clippy::doc_link_code)]
    /// Shortcut for [`.parser()`](Cell::parser)[`.parse()`](CellParser::parse)[`.ensure_empty()`](CellParser::ensure_empty).
    #[inline]
    pub fn parse_fully<'de, T>(&'de self, args: T::Args) -> Result<T, CellParserError<'de>>
    where
        T: CellDeserialize<'de>,
    {
        let mut parser = self.parser();
        let v = parser.parse(args)?;
        parser.ensure_empty()?;
        Ok(v)
    }

    #[allow(clippy::doc_link_code)]
    /// Shortcut for [`.parser()`](Cell::parser)[`.parse_as()`](CellParser::parse_as)[`.ensure_empty()`](CellParser::ensure_empty).
    #[inline]
    pub fn parse_fully_as<'de, T, As>(&'de self, args: As::Args) -> Result<T, CellParserError<'de>>
    where
        As: CellDeserializeAs<'de, T> + ?Sized,
    {
        let mut parser = self.parser();
        let v = parser.parse_as::<T, As>(args)?;
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

    /// See [Cell level](https://docs.ton.org/blockchain-basics/primitives/serialization/cells#level-of-a-cell)
    #[inline]
    pub(super) fn level_mask_with<'a>(
        &self,
        child_masks: impl IntoIterator<Item = &'a LevelMask>,
    ) -> LevelMask {
        let kind = self.exotic_kind();
        if kind.is_some_and(|k| k.is_pruned_branch()) {
            return LevelMask::new(self.data.as_raw_slice()[1]);
        }

        let mask = child_masks
            .into_iter()
            .fold(LevelMask::default(), |acc, m| acc | *m);

        if kind.is_some_and(|k| k.is_merkle()) {
            mask.merkle_shift()
        } else {
            mask
        }
    }

    /// [Standard Cell representation hash](https://docs.ton.org/blockchain-basics/primitives/serialization/cells#standard-cell-representation-and-its-hash)
    #[inline]
    pub fn hash_digest<D>(&self) -> [u8; 32]
    where
        D: Digest,
        Output<D>: Into<[u8; 32]>,
    {
        let hasher = CellHasher::<D>::new();
        hasher.repr_hash(self)
    }

    /// Calculates [standard Cell representation hash](https://docs.ton.org/blockchain-basics/primitives/serialization/cells#standard-cell-representation-and-its-hash)
    #[cfg(feature = "sha2")]
    #[inline]
    pub fn hash(&self) -> [u8; 32] {
        let hasher = CellHasher::<sha2::Sha256>::new();

        hasher.repr_hash(self)
    }

    #[cfg(feature = "sha2")]
    #[inline]
    pub fn level_hash(&self, level: u8) -> (u16, [u8; 32]) {
        let hasher = CellHasher::<sha2::Sha256>::new();

        hasher.level_hash(self, level)
    }

    /// Iterate this cell's DAG in DFS post-order
    /// (descendants left-to-right, then the cell). See [`CellIter`].
    #[inline]
    pub fn iter(&self) -> CellIter<'_> {
        CellIter::new(self)
    }

    pub(crate) fn exotic_kind(&self) -> Option<ExoticCellKind> {
        if !self.is_exotic {
            return None;
        }

        let data = self.data.as_raw_slice();
        let kind = match data.first()? {
            0x01 if data.len() == 36 || data.len() == 70 || data.len() == 104 => {
                ExoticCellKind::PrunedBranch
            }
            0x02 if data.len() == 33 => ExoticCellKind::LibraryReference,
            0x03 if data.len() == 35 => ExoticCellKind::MerkleProof,
            0x04 if data.len() == 69 => ExoticCellKind::MerkleUpdate,
            tag => ExoticCellKind::Unknown { tag: *tag },
        };

        Some(kind)
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

#[cfg(feature = "arbitrary")]
const _: () = {
    use arbitrary::{Arbitrary, MaxRecursionReached, Result, Unstructured, size_hint};
    use bitvec::mem::bits_of;

    use crate::ser::{MAX_BITS_LEN, MAX_REFS_COUNT};

    impl<'a> Arbitrary<'a> for Cell {
        fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
            Ok(Self {
                is_exotic: false,
                data: {
                    let len_bytes = u
                        .arbitrary_len::<u8>()?
                        .min(MAX_BITS_LEN.div_ceil(bits_of::<u8>()));
                    let bytes = u.bytes(len_bytes)?;
                    let mut bits = BitVec::from_slice(bytes);
                    bits.truncate(MAX_BITS_LEN);
                    bits
                },
                references: u
                    .arbitrary_iter()?
                    .take(MAX_REFS_COUNT)
                    .collect::<Result<_>>()?,
            })
        }

        #[inline]
        fn size_hint(depth: usize) -> (usize, Option<usize>) {
            Self::try_size_hint(depth).unwrap_or_default()
        }

        fn try_size_hint(depth: usize) -> Result<(usize, Option<usize>), MaxRecursionReached> {
            size_hint::try_recursion_guard(depth, |depth| {
                Ok(size_hint::and(
                    (0, Some(MAX_BITS_LEN.div_ceil(bits_of::<u8>()))),
                    <Vec<Arc<Self>> as Arbitrary>::size_hint(depth),
                ))
            })
        }

        fn arbitrary_take_rest(mut u: Unstructured<'a>) -> Result<Self> {
            Ok(Self {
                is_exotic: false,
                data: {
                    let len_bytes = u.len().min(MAX_BITS_LEN.div_ceil(bits_of::<u8>()));
                    let bytes = u.bytes(len_bytes)?;
                    let mut bits = BitVec::from_slice(bytes);
                    bits.truncate(MAX_BITS_LEN);
                    bits
                },
                references: u
                    .arbitrary_take_rest_iter()?
                    .take(MAX_REFS_COUNT)
                    .collect::<Result<_>>()?,
            })
        }
    }
};

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use crate::{
        r#as::{Data, Ref},
        bits::{NBits, NoArgs, ser::BitWriterExt},
        ser::CellSerializeExt,
        tests::assert_store_parse_as_eq,
    };

    use super::*;

    #[test]
    fn cell_serde() {
        assert_store_parse_as_eq::<
            _,
            (
                Data<NBits<1>>,
                Ref<Data<NBits<24>>>,
                Ref<(Data<NBits<7>>, Ref<Data<NBits<24>>>)>,
            ),
        >((0b1, 0x0AAAAA, (0x7F, 0x0AAAAA)), NoArgs::EMPTY);
    }

    #[test]
    fn hash_no_refs() {
        let mut builder = Cell::builder();
        builder.pack_as::<_, NBits<32>>(0x0000000F, ()).unwrap();
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
            .store_as::<_, Data<NBits<24>>>(0x00000B, ())
            .unwrap()
            .store_reference_as::<_, Data>(0x0000000F_u32, ())
            .unwrap()
            .store_reference_as::<_, Data>(0x0000000F_u32, ())
            .unwrap();
        let cell = builder.into_cell();

        assert_eq!(
            cell.hash(),
            hex!("f345277cc6cfa747f001367e1e873dcfa8a936b8492431248b7a3eeafa8030e7")
        );
    }

    #[test]
    fn cell_exotic_serde() {
        let expected = Cell {
            is_exotic: true,
            data: BitVec::from_slice(&[0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
            references: vec![Arc::new(Cell {
                is_exotic: false,
                data: BitVec::from_slice(&[0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
                references: vec![],
            })],
        };

        let actual = expected
            .to_cell(NoArgs::EMPTY)
            .unwrap()
            .parse_fully::<Cell>(NoArgs::EMPTY)
            .unwrap();

        assert_eq!(actual, expected);
    }

    pub fn make_cell(name: u8, refs: Vec<Arc<Cell>>) -> Arc<Cell> {
        Arc::new(Cell {
            data: BitVec::from_vec(vec![name]),
            references: refs,
            ..Cell::default()
        })
    }

    pub fn cell_name(cell: &Cell) -> u8 {
        cell.data.as_raw_slice()[0]
    }

    pub fn make_tree() -> Arc<Cell> {
        let c = make_cell(b'C', vec![]);
        let d = make_cell(b'D', vec![]);
        let a = make_cell(b'A', vec![c]);
        let b = make_cell(b'B', vec![d]);
        make_cell(b'R', vec![a, b])
    }

    pub fn make_dag() -> Arc<Cell> {
        let c = make_cell(b'C', vec![]);
        let a = make_cell(b'A', vec![c.clone()]);
        let b = make_cell(b'B', vec![c]);
        make_cell(b'R', vec![a, b])
    }
}
