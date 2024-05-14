use impl_tools::autoimpl;
use tlb::{
    BitPack, BitReader, BitReaderExt, BitUnpack, BitWriter, BitWriterExt, Cell, CellBuilder,
    CellBuilderError, CellDeserialize, CellParser, CellParserError, CellSerialize, NBits, Ref,
};

#[derive(Debug, Clone, PartialEq, Eq)]
#[autoimpl(Default)]
/// _ split_depth:(Maybe (## 5)) special:(Maybe TickTock)
/// code:(Maybe ^Cell) data:(Maybe ^Cell)
/// library:(Maybe ^Cell) = StateInit;
pub struct StateInit<C = Cell, D = Cell, L = Cell> {
    pub split_depth: Option<u8>,
    pub special: Option<TickTock>,
    pub code: Option<C>,
    pub data: Option<D>,
    pub library: Option<L>,
}

impl<C, D, L> CellSerialize for StateInit<C, D, L>
where
    C: CellSerialize,
    D: CellSerialize,
    L: CellSerialize,
{
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        builder
            .pack_as::<_, Option<NBits<5>>>(self.split_depth)?
            .pack(self.special)?
            .store_as::<_, Option<Ref>>(self.code.as_ref())?
            .store_as::<_, Option<Ref>>(self.data.as_ref())?
            .store_as::<_, Option<Ref>>(self.library.as_ref())?;
        Ok(())
    }
}

impl<'de, C, D, L> CellDeserialize<'de> for StateInit<C, D, L>
where
    C: CellDeserialize<'de>,
    D: CellDeserialize<'de>,
    L: CellDeserialize<'de>,
{
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, CellParserError<'de>> {
        Ok(Self {
            split_depth: parser.unpack_as::<_, Option<NBits<5>>>()?,
            special: parser.unpack()?,
            code: parser.parse_as::<_, Option<Ref>>()?,
            data: parser.parse_as::<_, Option<Ref>>()?,
            library: parser.parse_as::<_, Option<Ref>>()?,
        })
    }
}

/// tick_tock$_ tick:Bool tock:Bool = TickTock;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TickTock {
    tick: bool,
    tock: bool,
}

impl BitPack for TickTock {
    fn pack<W>(&self, mut writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        writer.pack(self.tick)?.pack(self.tock)?;
        Ok(())
    }
}

impl BitUnpack for TickTock {
    fn unpack<R>(mut reader: R) -> Result<Self, R::Error>
    where
        R: BitReader,
    {
        Ok(Self {
            tick: reader.unpack()?,
            tock: reader.unpack()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use tlb::CellSerializeExt;

    use super::*;

    #[test]
    fn state_init_serde() {
        let s = StateInit::<(), (), ()>::default();
        let cell = s.to_cell().unwrap();
        let got: StateInit<(), (), ()> = cell.parse_fully().unwrap();
        assert_eq!(got, s);
    }
}
