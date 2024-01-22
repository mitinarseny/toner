use core::{
    fmt::{self, Debug},
    hash::Hash,
    ops::Deref,
};
use std::sync::Arc;

use bitvec::{
    order::{BitOrder, Msb0},
    slice::BitSlice,
    store::BitStore,
    vec::BitVec,
    view::AsBits,
};
use sha2::{Digest, Sha256};

use crate::{serialize::TLBSerialize, CellBuilder, CellParser, ErrorReason, Result};

const MAX_BITS_LEN: usize = 1023;
const MAX_REFS_COUNT: usize = 4;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Cell {
    data: BitVec<u8, Msb0>,
    references: Vec<Arc<Self>>,
}

impl Cell {
    #[inline]
    #[must_use]
    pub const fn builder() -> CellBuilder {
        CellBuilder::new()
    }

    pub fn parser(&self) -> CellParser<'_> {
        CellParser::new(&self.data, &self.references)
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
    pub fn is_empty(&self) -> bool {
        self.data.is_empty() && self.references().is_empty()
    }

    #[inline]
    pub fn data(&self) -> &BitSlice<u8, Msb0> {
        &self.data
    }

    #[inline]
    pub fn data_bytes(&self) -> (usize, &[u8]) {
        (self.bits_len(), self.data.as_raw_slice())
    }

    #[inline]
    pub fn bits_len(&self) -> usize {
        self.data.len()
    }

    #[inline]
    pub fn reference(&self, index: usize) -> Option<&Arc<Self>> {
        self.references().get(index)
    }

    #[inline]
    pub fn references(&self) -> &[Arc<Self>] {
        &self.references
    }

    #[inline]
    pub fn has_references(&self) -> bool {
        !self.references().is_empty()
    }

    #[inline]
    pub fn push_bit(&mut self, bit: bool) -> Result<&mut Self> {
        if self.data.len() == MAX_BITS_LEN {
            return Err(ErrorReason::TooLong.into());
        }
        self.data.push(bit);
        Ok(self)
    }

    #[inline]
    pub fn push_bits<T, O>(&mut self, bits: impl AsRef<BitSlice<T, O>>) -> Result<&mut Self>
    where
        T: BitStore,
        O: BitOrder,
    {
        let bits = bits.as_ref();
        if self.bits_len() + bits.len() > MAX_BITS_LEN {
            return Err(ErrorReason::TooLong.into());
        }
        self.data.extend_from_bitslice(bits);
        Ok(self)
    }

    #[inline]
    pub fn repeat_bit(&mut self, n: usize, bit: bool) -> Result<&mut Self> {
        if self.bits_len() + n > MAX_BITS_LEN {
            return Err(ErrorReason::TooLong.into());
        }
        self.data.resize(self.bits_len() + n, bit);
        Ok(self)
    }

    #[inline]
    pub fn push_bytes<T>(&mut self, bytes: impl AsRef<[T]>) -> Result<&mut Self>
    where
        T: BitStore,
    {
        self.push_bits(bytes.as_bits::<Msb0>())
    }

    #[inline]
    pub fn push_reference<T>(&mut self, reference: T) -> Result<&mut Self, T>
    where
        T: Into<Arc<Self>>,
    {
        if self.references.len() == MAX_REFS_COUNT {
            return Err(reference);
        }
        self.references.push(reference.into());
        Ok(self)
    }

    pub fn extend_references<T>(
        &mut self,
        references: impl IntoIterator<Item = impl Into<Arc<Self>>>,
    ) -> Result<&mut Self> {
        for r in references {
            self.push_reference(r)
                .map_err(|_| ErrorReason::TooManyReferences)?;
        }
        Ok(self)
    }

    /// See [Cell level](https://docs.ton.org/develop/data-formats/cell-boc#cell-level)
    #[inline]
    fn level(&self) -> u8 {
        self.references()
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
        self.references().len() as u8 | (self.level() << 5)
    }

    /// See [Cell serialization](https://docs.ton.org/develop/data-formats/cell-boc#cell-serialization)
    #[inline]
    fn bits_descriptor(&self) -> u8 {
        let b = self.bits_len();
        (b / 8) as u8 + ((b + 7) / 8) as u8
    }

    fn max_depth(&self) -> u16 {
        self.references()
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
        buf.push(self.refs_descriptor());
        buf.push(self.bits_descriptor());

        let rest_bits = self.bits_len() % 8;

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
            self.references()
                .iter()
                .flat_map(|r| r.max_depth().to_be_bytes()),
        );

        // refs hashes
        buf.extend(
            self.references()
                .iter()
                .map(Deref::deref)
                .flat_map(Cell::hash),
        );

        buf
    }

    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(self.repr());
        hasher.finalize().into()
    }

    pub fn serialize(&self) -> Vec<u8> {
        todo!()
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self::new()
    }
}

impl TLBSerialize for Cell {
    fn store(&self, builder: &mut CellBuilder) -> Result<()> {
        builder
            .store(self.data.as_bitslice())?
            .store(self.references())?;
        Ok(())
    }
}

impl Debug for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (bits_len, data) = self.data_bytes();
        write!(f, "{}[0x{}]", bits_len, hex::encode_upper(data))?;
        if !self.has_references() {
            return Ok(());
        }
        write!(f, " -> ")?;
        f.debug_set().entries(self.references()).finish()
    }
}

#[cfg(test)]
mod tests {
    use bitvec::{bitvec, order::Msb0, view::BitViewSized};
    use hex_literal::hex;

    use super::*;
    use crate::{NBits, Ref, TLBSerializeExt, TLBSerializeWrapAs};

    #[test]
    fn zero_depth() {
        assert_eq!(().to_cell().unwrap().max_depth(), 0)
    }

    #[test]
    fn max_depth() {
        assert_eq!(
            ((), ((), ()), ((), ()))
                .wrap_as::<(Ref, Ref<Ref<(Ref, Ref<Ref>)>>, Ref<(Ref, Ref<Ref>)>)>()
                .to_cell()
                .unwrap()
                .max_depth(),
            4
        )
    }

    #[test]
    fn tlb_serialize() {
        assert_eq!(
            (
                &hex!("80").as_bits::<Msb0>()[..1],
                0x0AAAAA.wrap_as::<NBits<24>>().wrap_as::<Ref>(),
                (
                    &hex!("FD").as_bits::<Msb0>()[..7],
                    0x0AAAAA.wrap_as::<NBits<24>>().wrap_as::<Ref>(),
                )
                    .wrap_as::<Ref>(),
            )
                .to_cell()
                .unwrap(),
            Cell {
                data: bitvec![u8, Msb0; 1],
                references: [
                    Cell {
                        data: hex!("0AAAAA").into_bitarray().into(),
                        references: [].into()
                    },
                    Cell {
                        data: bitvec![u8, Msb0; 1, 1, 1, 1, 1, 1, 0],
                        references: [Cell {
                            data: hex!("0AAAAA").into_bitarray().into(),
                            references: [].into()
                        }]
                        .map(Into::into)
                        .into(),
                    }
                ]
                .map(Into::into)
                .into()
            },
        );
    }

    #[test]
    fn tlb_serde() {
        type As = (NBits<1>, Ref<NBits<24>>, Ref<(NBits<7>, Ref<NBits<24>>)>);
        let cell = (0b1, 0x0AAAAA, (0x7F, 0x0AAAAA));

        assert_eq!(
            cell,
            cell.wrap_as::<As>()
                .to_cell()
                .unwrap()
                .parser()
                .parse_fully_as::<_, As>()
                .unwrap(),
        )
    }

    #[test]
    fn cell_serialize() {
        let cell = (
            0b1.wrap_as::<NBits<1>>(),
            0x0AAAAA.wrap_as::<NBits<24>>().wrap_as::<Ref>(),
            (
                0x7F.wrap_as::<NBits<7>>(),
                0x0AAAAA.wrap_as::<NBits<24>>().wrap_as::<Ref>(),
            )
                .wrap_as::<Ref>(),
        )
            .to_cell()
            .unwrap();
        assert_eq!(cell.serialize(), hex!("0201c002010101ff0200060aaaaa"));
    }

    #[test]
    fn hash_no_refs() {
        let cell = 0x0000000F.wrap_as::<NBits<32>>().to_cell().unwrap();

        assert_eq!(
            cell.hash(),
            hex!("57b520dbcb9d135863fc33963cde9f6db2ded1430d88056810a2c9434a3860f9")
        );
    }

    #[test]
    fn hash_with_refs() {
        let cell = (
            0x00000B.wrap_as::<NBits<24>>(),
            0x0000000F_u32.wrap_as::<Ref>(),
            0x0000000F_u32.wrap_as::<Ref>(),
        )
            .to_cell()
            .unwrap();

        assert_eq!(
            cell.hash(),
            hex!("f345277cc6cfa747f001367e1e873dcfa8a936b8492431248b7a3eeafa8030e7")
        );
    }
}
