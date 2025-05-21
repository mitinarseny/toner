//! Collection of types related to [StateInit](https://docs.ton.org/develop/data-formats/msg-tlb#stateinit-tl-b)
use impl_tools::autoimpl;
use tlb::{
    Cell,
    r#as::{NoArgs, ParseFully, Ref, hashmap::HashmapE},
    bits::{
        r#as::NBits,
        de::{BitReader, BitReaderExt, BitUnpack},
        ser::{BitPack, BitWriter, BitWriterExt},
    },
    de::{CellDeserialize, CellParser, CellParserError},
    ser::{CellBuilder, CellBuilderError, CellSerialize, CellSerializeExt},
};

/// [StateInit](https://docs.ton.org/develop/data-formats/msg-tlb#stateinit-tl-b)
/// ```tlb
/// _ split_depth:(Maybe (## 5)) special:(Maybe TickTock)
/// code:(Maybe ^Cell) data:(Maybe ^Cell)
/// library:(HashmapE 256 SimpleLib) = StateInitWithLibs;
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[autoimpl(Default)]
pub struct StateInit<C = Cell, D = Cell> {
    pub split_depth: Option<u8>,
    pub special: Option<TickTock>,
    pub code: Option<C>,
    pub data: Option<D>,
    pub library: HashmapE<SimpleLib>,
}

impl<IC, ID> StateInit<IC, ID>
where
    IC: CellSerialize,
    ID: CellSerialize,
{
    #[inline]
    pub fn normalize(&self) -> Result<StateInit, CellBuilderError> {
        Ok(StateInit {
            split_depth: self.split_depth,
            special: self.special,
            code: self
                .code
                .as_ref()
                .map(CellSerializeExt::to_cell)
                .transpose()?,
            data: self
                .data
                .as_ref()
                .map(CellSerializeExt::to_cell)
                .transpose()?,
            library: self.library.clone(),
        })
    }
}

impl<C, D> CellSerialize for StateInit<C, D>
where
    C: CellSerialize,
    D: CellSerialize,
{
    #[inline]
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        builder
            // split_depth:(Maybe (## 5))
            .pack_as::<_, Option<NBits<5>>>(self.split_depth)?
            // special:(Maybe TickTock)
            .pack(self.special)?
            // code:(Maybe ^Cell)
            .store_as::<_, Option<Ref>>(self.code.as_ref())?
            // data:(Maybe ^Cell)
            .store_as::<_, Option<Ref>>(self.data.as_ref())?
            // library:(HashmapE 256 SimpleLib)
            .store_as_with::<_, &HashmapE<NoArgs<_>, NoArgs<_>>>(&self.library, (256, (), ()))?;
        Ok(())
    }
}

impl<'de, C, D> CellDeserialize<'de> for StateInit<C, D>
where
    C: CellDeserialize<'de>,
    D: CellDeserialize<'de>,
{
    #[inline]
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, CellParserError<'de>> {
        Ok(Self {
            // split_depth:(Maybe (## 5))
            split_depth: parser.unpack_as::<_, Option<NBits<5>>>()?,
            // special:(Maybe TickTock)
            special: parser.unpack()?,
            // code:(Maybe ^Cell)
            code: parser.parse_as::<_, Option<Ref<ParseFully>>>()?,
            // data:(Maybe ^Cell)
            data: parser.parse_as::<_, Option<Ref<ParseFully>>>()?,
            // library:(HashmapE 256 SimpleLib)
            library: parser.parse_as_with::<_, HashmapE<NoArgs<_>, NoArgs<_>>>((256, (), ()))?,
        })
    }
}

/// `tick_tock` field for [`StateInit`]
/// ```tlb
/// tick_tock$_ tick:Bool tock:Bool = TickTock;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TickTock {
    pub tick: bool,
    pub tock: bool,
}

impl BitPack for TickTock {
    #[inline]
    fn pack<W>(&self, mut writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        writer.pack(self.tick)?.pack(self.tock)?;
        Ok(())
    }
}

impl BitUnpack for TickTock {
    #[inline]
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

/// `library` field for [`StateInit`]
/// ```tlb
/// simple_lib$_ public:Bool root:^Cell = SimpleLib;
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleLib {
    pub public: bool,
    pub root: Cell,
}

impl CellSerialize for SimpleLib {
    #[inline]
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        builder.pack(self.public)?.store_as::<_, Ref>(&self.root)?;
        Ok(())
    }
}

impl<'de> CellDeserialize<'de> for SimpleLib {
    #[inline]
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, CellParserError<'de>> {
        Ok(SimpleLib {
            public: parser.unpack()?,
            root: parser.parse_as::<_, Ref>()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use tlb::ser::CellSerializeExt;

    use super::*;

    #[test]
    fn state_init_serde() {
        let s = StateInit::<(), ()>::default();
        let cell = s.to_cell().unwrap();
        let got: StateInit<(), ()> = cell.parse_fully().unwrap();
        assert_eq!(got, s);
    }
}
