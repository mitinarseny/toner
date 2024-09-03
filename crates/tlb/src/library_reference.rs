use tlbits::de::BitReaderExt;
use tlbits::ser::BitWriterExt;
use crate::cell_type::CellType;
use crate::de::{CellDeserialize, CellParser, CellParserError};
use crate::ser::{CellBuilder, CellBuilderError, CellSerialize};

pub struct LibraryReference {
    pub hash: [u8; 32]
}

impl CellSerialize for LibraryReference {
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        builder.set_type(CellType::LibraryReference);
        builder.pack(self.hash)?;

        Ok(())
    }
}

impl<'de> CellDeserialize<'de> for LibraryReference {
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, CellParserError<'de>> {
        parser.ensure_type(CellType::LibraryReference)?;
        let hash = parser.unpack()?;

        Ok(LibraryReference { hash })
    }
}
