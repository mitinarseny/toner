use std::fmt::{Debug, Display, Formatter};
use strum::FromRepr;
use tlbits::de::{BitReader, BitReaderExt, BitUnpack};
use tlbits::Error;
use tlbits::ser::{BitPack, BitWriter, BitWriterExt};

#[repr(u8)]
#[derive(Clone, PartialEq, Eq, Hash, Copy, Debug, FromRepr)]
pub enum CellType {
    Ordinary = 255_u8,
    PrunedBranch = 1_u8,
    LibraryReference = 2_u8,
    MerkleProof = 3_u8,
    MerkleUpdate = 4_u8,
}

impl Display for CellType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl Default for CellType {
    fn default() -> Self {
        Self::Ordinary
    }
}

impl BitUnpack for CellType {
    fn unpack<R>(mut reader: R) -> Result<Self, R::Error>
    where
        R: BitReader,
    {
        let raw_type = reader.unpack()?;

        CellType::from_repr(raw_type)
            .ok_or_else(|| Error::custom(format!("unknown cell type: {}", raw_type)))
    }
}

impl BitPack for CellType {
    fn pack<W>(&self, mut writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        writer.pack(*self as u8)?;

        Ok(())
    }
}

impl CellType {
    pub fn is_exotic(&self) -> bool {
        !matches!(self, Self::Ordinary)
    }
}
