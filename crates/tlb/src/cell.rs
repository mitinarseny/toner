use core::{
    fmt::{self, Debug},
    hash::Hash,
    ops::Deref,
};
use std::sync::Arc;

use bitvec::{order::Msb0, vec::BitVec};
use sha2::{Digest, Sha256};

use crate::{CellBuilder, CellDeserialize, CellDeserializeAs, CellParser, CellParserError};

#[derive(Clone, Default, PartialEq, Eq, Hash)]
pub struct Cell {
    pub data: BitVec<u8, Msb0>,
    pub references: Vec<Arc<Self>>,
}

impl Cell {
    #[inline]
    #[must_use]
    pub const fn builder() -> CellBuilder {
        CellBuilder::new()
    }

    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            data: BitVec::EMPTY,
            references: Vec::new(),
        }
    }

    #[inline]
    pub fn parser(&self) -> CellParser<'_> {
        CellParser::new(&self.data, &self.references)
    }

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
    fn level(&self) -> u8 {
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
        (b / 8) as u8 + ((b + 7) / 8) as u8
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
    fn repr(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        // buf.pack(self.refs_descriptor())
        buf.push(self.refs_descriptor());
        buf.push(self.bits_descriptor());

        let rest_bits = self.data.len() % 8;

        if rest_bits == 0 {
            buf.extend(self.data.as_raw_slice());
        } else {
            let (last, data) = self.data.as_raw_slice().split_last().unwrap();
            buf.extend(data);
            let mut last = last & !(!0u8 << rest_bits); // clear the rest
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

    #[inline]
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(self.repr());
        hasher.finalize().into()
    }

    // pub fn serialize(&self) -> Vec<u8> {
    //     todo!()
    // }
}

impl Debug for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (bits_len, data) = self.data_bytes();
        write!(f, "{}[0x{}]", bits_len, hex::encode_upper(data))?;
        if self.references.is_empty() {
            return Ok(());
        }
        write!(f, " -> ")?;
        f.debug_set().entries(&self.references).finish()
    }
}

#[cfg(feature = "tonlib")]
mod tonlib {
    use ::tonlib::cell::Cell as TonlibCell;
    use bitvec::view::AsBits;

    use super::*;

    impl From<&Cell> for TonlibCell {
        fn from(cell: &Cell) -> Self {
            Self {
                data: cell.data.clone().into_vec(),
                bit_len: cell.data.len(),
                references: cell
                    .references
                    .iter()
                    .map(Deref::deref)
                    .map(Into::into)
                    .map(Arc::new)
                    .collect(),
            }
        }
    }

    impl From<&TonlibCell> for Cell {
        fn from(cell: &TonlibCell) -> Self {
            Self {
                data: BitVec::from_bitslice(&cell.data.as_bits()[..cell.bit_len]),
                references: cell
                    .references
                    .iter()
                    .map(Deref::deref)
                    .map(Into::into)
                    .map(Arc::new)
                    .collect(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{BitWriterExt, NBits};
    use hex_literal::hex;

    use crate::{
        tests::assert_store_parse_as_eq, CellSerializeExt, CellSerializeWrapAsExt, Data, Ref,
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

    // #[test]
    // fn cell_serialize() {
    //     assert_eq!(
    //         (
    //             &hex!("80").as_bits::<Msb0>()[..1],
    //             0x0AAAAA.wrap_as::<NBits<24>>().wrap_as::<Ref>(),
    //             (
    //                 &hex!("FD").as_bits::<Msb0>()[..7],
    //                 0x0AAAAA.wrap_as::<NBits<24>>().wrap_as::<Ref>(),
    //             )
    //                 .wrap_as::<Ref>(),
    //         )
    //             .to_cell()
    //             .unwrap(),
    //         Cell {
    //             data: bitvec![u8, Msb0; 1],
    //             references: [
    //                 Cell {
    //                     data: hex!("0AAAAA").into_bitarray().into(),
    //                     references: [].into()
    //                 },
    //                 Cell {
    //                     data: bitvec![u8, Msb0; 1, 1, 1, 1, 1, 1, 0],
    //                     references: [Cell {
    //                         data: hex!("0AAAAA").into_bitarray().into(),
    //                         references: [].into()
    //                     }]
    //                     .map(Into::into)
    //                     .into(),
    //                 }
    //             ]
    //             .map(Into::into)
    //             .into()
    //         },
    //     );
    // }

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

    //     #[test]
    //     #[ignore = "wait until serialize is implemented"]
    //     fn cell_serialize() {
    //         let cell = (
    //             0b1.wrap_as::<NBits<1>>(),
    //             0x0AAAAA.wrap_as::<NBits<24>>().wrap_as::<Ref>(),
    //             (
    //                 0x7F.wrap_as::<NBits<7>>(),
    //                 0x0AAAAA.wrap_as::<NBits<24>>().wrap_as::<Ref>(),
    //             )
    //                 .wrap_as::<Ref>(),
    //         )
    //             .to_cell()
    //             .unwrap();
    //         assert_eq!(cell.serialize(), hex!("0201c002010101ff0200060aaaaa"));
    //     }
}
