use crate::cell_type::CellType;
use crate::de::{CellDeserialize, CellParser, CellParserError};
use crate::ser::{CellBuilder, CellBuilderError, CellSerialize};
use tlbits::de::BitReaderExt;
use tlbits::ser::BitWriterExt;

#[derive(Clone, Default, PartialEq, Eq, Hash)]
pub struct LibraryReferenceCell {
    pub hash: [u8; 32],
}

impl<'de> CellSerialize for LibraryReferenceCell {
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        builder.set_type(CellType::LibraryReference);
        builder.pack(self.hash)?;

        Ok(())
    }
}

impl<'de> CellDeserialize<'de> for LibraryReferenceCell {
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, CellParserError<'de>> {
        parser.ensure_type(CellType::LibraryReference)?;
        let hash = parser.unpack()?;
        parser.ensure_empty()?;

        Ok(Self { hash })
    }
}
