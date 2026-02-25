//! Collection of types related to [StateInit](https://docs.ton.org/develop/data-formats/msg-tlb#stateinit-tl-b)
use impl_tools::autoimpl;
use tlb::{
    Cell, ParseFully, Ref, Same,
    bits::{
        NBits, NoArgs,
        de::{BitReader, BitReaderExt, BitUnpack},
        ser::{BitPack, BitWriter, BitWriterExt},
    },
    de::{CellDeserialize, CellParser, CellParserError},
    hashmap::HashmapE,
    ser::{CellBuilder, CellBuilderError, CellSerialize, CellSerializeExt},
};

/// [StateInit](https://docs.ton.org/develop/data-formats/msg-tlb#stateinit-tl-b)
/// ```tlb
/// _ split_depth:(Maybe (## 5)) special:(Maybe TickTock)
/// code:(Maybe ^Cell) data:(Maybe ^Cell)
/// library:(HashmapE 256 SimpleLib) = StateInitWithLibs;
/// ```
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq)]
#[autoimpl(Default)]
pub struct StateInit<C = Cell, D = Cell> {
    pub split_depth: Option<u8>,
    pub special: Option<TickTock>,
    pub code: Option<C>,
    pub data: Option<D>,
    #[cfg_attr(feature = "arbitrary", arbitrary(default))] // TODO
    pub library: HashmapE<SimpleLib>,
}

impl<IC, ID> StateInit<IC, ID>
where
    IC: CellSerialize<Args: NoArgs>,
    ID: CellSerialize<Args: NoArgs>,
{
    #[inline]
    pub fn normalize(&self) -> Result<StateInit, CellBuilderError> {
        Ok(StateInit {
            split_depth: self.split_depth,
            special: self.special,
            code: self
                .code
                .as_ref()
                .map(|c| c.to_cell(NoArgs::EMPTY))
                .transpose()?,
            data: self
                .data
                .as_ref()
                .map(|d| d.to_cell(NoArgs::EMPTY))
                .transpose()?,
            library: self.library.clone(),
        })
    }
}

impl<C, D> CellSerialize for StateInit<C, D>
where
    C: CellSerialize<Args: NoArgs>,
    D: CellSerialize<Args: NoArgs>,
{
    type Args = ();

    #[inline]
    fn store(&self, builder: &mut CellBuilder, _: Self::Args) -> Result<(), CellBuilderError> {
        builder
            // split_depth:(Maybe (## 5))
            .pack_as::<_, Option<NBits<5>>>(self.split_depth, ())?
            // special:(Maybe TickTock)
            .pack(self.special, ())?
            // code:(Maybe ^Cell)
            .store_as::<_, Option<Ref>>(self.code.as_ref(), NoArgs::EMPTY)?
            // data:(Maybe ^Cell)
            .store_as::<_, Option<Ref>>(self.data.as_ref(), NoArgs::EMPTY)?
            // library:(HashmapE 256 SimpleLib)
            .store_as::<_, &HashmapE<Same, Same>>(&self.library, (256, (), ()))?;
        Ok(())
    }
}

impl<'de, C, D> CellDeserialize<'de> for StateInit<C, D>
where
    C: CellDeserialize<'de, Args: NoArgs>,
    D: CellDeserialize<'de, Args: NoArgs>,
{
    type Args = ();

    #[inline]
    fn parse(parser: &mut CellParser<'de>, _: Self::Args) -> Result<Self, CellParserError<'de>> {
        Ok(Self {
            // split_depth:(Maybe (## 5))
            split_depth: parser.unpack_as::<_, Option<NBits<5>>>(())?,
            // special:(Maybe TickTock)
            special: parser.unpack(())?,
            // code:(Maybe ^Cell)
            code: parser.parse_as::<_, Option<Ref<ParseFully>>>(NoArgs::EMPTY)?,
            // data:(Maybe ^Cell)
            data: parser.parse_as::<_, Option<Ref<ParseFully>>>(NoArgs::EMPTY)?,
            // library:(HashmapE 256 SimpleLib)
            library: parser.parse_as::<_, HashmapE<Same, Same>>((256, (), ()))?,
        })
    }
}

/// `tick_tock` field for [`StateInit`]
/// ```tlb
/// tick_tock$_ tick:Bool tock:Bool = TickTock;
/// ```
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TickTock {
    pub tick: bool,
    pub tock: bool,
}

impl BitPack for TickTock {
    type Args = ();

    #[inline]
    fn pack<W>(&self, writer: &mut W, _: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        writer.pack(self.tick, ())?.pack(self.tock, ())?;
        Ok(())
    }
}

impl<'de> BitUnpack<'de> for TickTock {
    type Args = ();

    #[inline]
    fn unpack<R>(reader: &mut R, _: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        Ok(Self {
            tick: reader.unpack(())?,
            tock: reader.unpack(())?,
        })
    }
}

/// `library` field for [`StateInit`]
/// ```tlb
/// simple_lib$_ public:Bool root:^Cell = SimpleLib;
/// ```
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleLib {
    pub public: bool,
    pub root: Cell,
}

impl CellSerialize for SimpleLib {
    type Args = ();

    #[inline]
    fn store(&self, builder: &mut CellBuilder, _: Self::Args) -> Result<(), CellBuilderError> {
        builder
            .pack(self.public, ())?
            .store_as::<_, Ref>(&self.root, ())?;
        Ok(())
    }
}

impl<'de> CellDeserialize<'de> for SimpleLib {
    type Args = ();

    #[inline]
    fn parse(parser: &mut CellParser<'de>, _: Self::Args) -> Result<Self, CellParserError<'de>> {
        Ok(SimpleLib {
            public: parser.unpack(())?,
            root: parser.parse_as::<_, Ref>(())?,
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
        let cell = s.to_cell(()).unwrap();
        let got: StateInit<(), ()> = cell.parse_fully(()).unwrap();
        assert_eq!(got, s);
    }
}
