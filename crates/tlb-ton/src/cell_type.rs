use strum::FromRepr;
use tlb::bits::de::{BitReader, BitReaderExt, BitUnpack};
use tlb::bits::ser::{BitPack, BitWriter, BitWriterExt};
use tlb::cell_type::CellType;
use tlb::Error;

/// Types of [OrdinaryCell] (https://docs.ton.org/develop/data-formats/exotic-cells).
#[repr(u8)]
#[derive(PartialEq, Eq, Debug, Clone, Hash, Copy, FromRepr)]
pub enum RawCellType {
    Ordinary = 255_u8,
    PrunedBranch = 1_u8,
    LibraryReference = 2_u8,
    MerkleProof = 3_u8,
    MerkleUpdate = 4_u8,
}

impl From<RawCellType> for CellType {
    fn from(value: RawCellType) -> CellType {
        match value {
            RawCellType::Ordinary => CellType::Ordinary,
            RawCellType::PrunedBranch => CellType::PrunedBranch,
            RawCellType::LibraryReference => CellType::LibraryReference,
            RawCellType::MerkleProof => CellType::MerkleProof,
            RawCellType::MerkleUpdate => CellType::MerkleUpdate,
        }
    }
}

impl From<CellType> for RawCellType {
    fn from(value: CellType) -> Self {
        match value {
            CellType::Ordinary => RawCellType::Ordinary,
            CellType::PrunedBranch => RawCellType::PrunedBranch,
            CellType::LibraryReference => RawCellType::LibraryReference,
            CellType::MerkleProof => RawCellType::MerkleProof,
            CellType::MerkleUpdate => RawCellType::MerkleUpdate,
        }
    }
}

impl BitUnpack for RawCellType {
    fn unpack<R>(mut reader: R) -> Result<Self, R::Error>
    where
        R: BitReader,
    {
        let raw_type = reader.unpack()?;

        RawCellType::from_repr(raw_type)
            .ok_or_else(|| Error::custom(format!("unknown cell type: {}", raw_type)))
    }
}

impl BitPack for RawCellType {
    fn pack<W>(&self, mut writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        writer.pack(*self as u8)?;

        Ok(())
    }
}

impl RawCellType {
    pub fn is_exotic(&self) -> bool {
        !matches!(self, Self::Ordinary)
    }
}
