use crate::cell_type::CellType;
use crate::de::{CellDeserialize, CellParser, CellParserError};
use crate::r#as::Ref;
use crate::ser::{CellBuilder, CellBuilderError, CellSerialize};
use crate::Cell;
use bitvec::order::Msb0;
use bitvec::prelude::BitVec;
use std::mem;
use std::sync::Arc;
use tlbits::ser::BitWriterExt;

#[derive(Clone, Default, PartialEq, Eq, Hash)]
pub struct OrdinaryCell {
    pub data: BitVec<u8, Msb0>,
    pub references: Vec<Arc<Cell>>,
}

impl CellSerialize for OrdinaryCell {
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        builder.set_type(CellType::Ordinary);

        builder
            .pack(self.data.as_bitslice())?
            .store_many_as::<_, Ref>(&self.references)?;

        Ok(())
    }
}

impl<'de> CellDeserialize<'de> for OrdinaryCell {
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, CellParserError<'de>> {
        parser.ensure_type(CellType::Ordinary)?;

        Ok(Self {
            data: mem::take(&mut parser.data).to_bitvec(),
            references: mem::take(&mut parser.references).to_vec(),
        })
    }
}
