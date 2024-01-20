use core::{
    fmt::Display,
    hash::{Hash, Hasher},
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
use impl_tools::autoimpl;
use sha2::{Digest, Sha256};

use crate::{serialize::TLBSerialize, CellBuilder, CellParser, Error, Result, TLBDeserialize};

const MAX_BITS_LEN: usize = 1023;
const MAX_REFS_COUNT: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq)]
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
    pub fn push_bit(&mut self, bit: bool) -> Result<&mut Self> {
        if self.data.len() == MAX_BITS_LEN {
            return Err(Error::TooLong);
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
            return Err(Error::TooLong);
        }
        self.data.extend_from_bitslice(bits);
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
                .map_err(|_| Error::TooManyReferences)?;
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
        // TODO
        Vec::new()
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self::new()
    }
}

impl Hash for Cell {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.repr().hash(state)
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

impl Display for Cell {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}{:b}", self.bits_len(), self.data)?;
        if self.references().is_empty() {
            return Ok(());
        }
        write!(f, " -> {{")?;
        for r in self.references() {
            r.fmt(f)?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}

#[autoimpl(Deref using self.0)]
#[autoimpl(DerefMut using self.0)]
#[autoimpl(AsRef using self.0)]
#[autoimpl(AsMut using self.0)]
#[derive(Debug, Clone, Copy)]
pub struct Ref<T>(pub T);

impl<T> TLBSerialize for Ref<T>
where
    T: TLBSerialize,
{
    #[inline]
    fn store(&self, builder: &mut CellBuilder) -> Result<()> {
        builder.store_reference(&self.0)?;
        Ok(())
    }
}

impl<T> TLBDeserialize for Ref<T>
where
    T: TLBDeserialize,
{
    fn parse(parser: &mut CellParser<'_>) -> Result<Self> {
        parser.parse_reference()
    }
}

#[cfg(test)]
mod tests {
    use bitvec::{bitvec, order::Msb0, view::BitViewSized};
    use hex_literal::hex;

    use super::*;
    use crate::{Num, TLBSerializeExt};

    #[test]
    fn zero_depth() {
        assert_eq!(().to_cell().unwrap().max_depth(), 0)
    }

    #[test]
    fn max_depth() {
        assert_eq!(
            (
                Ref(()),
                Ref(Ref((Ref(()), Ref(Ref(()))))),
                Ref((Ref(()), Ref(Ref(())))),
            )
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
                Ref(Num::<24, u32>(0x0AAAAA)),
                Ref((
                    &hex!("FD").as_bits::<Msb0>()[..7],
                    Ref(Num::<24, u32>(0x0AAAAA))
                )),
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
    fn cell_serialize() {
        let cell = (
            Num::<1, _>(0b1),
            Ref(Num::<24, _>(0x0AAAAA)),
            Ref((Num::<7, u8>(0x7F), Ref(Num::<24, _>(0x0AAAAA)))),
        )
            .to_cell()
            .unwrap();
        assert_eq!(cell.serialize(), hex!("0201c002010101ff0200060aaaaa"));
    }

    #[test]
    fn hash_no_refs() {
        let cell = Num::<32, u32>(0x0000000F).to_cell().unwrap();

        assert_eq!(
            cell.hash(),
            hex!("57b520dbcb9d135863fc33963cde9f6db2ded1430d88056810a2c9434a3860f9")
        );
    }

    #[test]
    fn hash_with_refs() {
        let cell = (
            Num::<24, u32>(0x00000B),
            Ref(0x0000000F_u32),
            Ref(0x0000000F_u32),
        )
            .to_cell()
            .unwrap();

        assert_eq!(
            cell.hash(),
            hex!("f345277cc6cfa747f001367e1e873dcfa8a936b8492431248b7a3eeafa8030e7")
        );
    }
}
