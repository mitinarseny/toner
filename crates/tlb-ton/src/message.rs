use impl_tools::autoimpl;
use num_bigint::BigUint;
use tlb::{
    BitPack, BitReader, BitReaderExt, BitUnpack, BitWriter, BitWriterExt, Cell, CellBuilder,
    CellBuilderError, CellDeserialize, CellParser, CellParserError, CellSerialize, Either, NBits,
    Ref, Same,
};

use crate::{CurrencyCollection, Grams, MsgAddress};

/// message$_ {X:Type} info:CommonMsgInfo
/// init:(Maybe (Either StateInit ^StateInit))
/// body:(Either X ^X) = Message X;
pub struct Message<T = Cell, IC = Cell, ID = Cell, IL = Cell> {
    pub info: CommonMsgInfo,
    pub init: Option<StateInit<IC, ID, IL>>,
    pub body: T,
}

impl<T, IC, ID, IL> CellSerialize for Message<T, IC, ID, IL>
where
    T: CellSerialize,
    IC: CellSerialize,
    ID: CellSerialize,
    IL: CellSerialize,
{
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        builder
            .pack(&self.info)?
            .store_as::<_, Option<Either<(), Ref>>>(self.init.as_ref().map(Some))?
            .store_as::<_, Ref>(&self.body)?;
        Ok(())
    }
}

impl<'de, T, IC, ID, IL> CellDeserialize<'de> for Message<T, IC, ID, IL>
where
    T: CellDeserialize<'de>,
    IC: CellDeserialize<'de>,
    ID: CellDeserialize<'de>,
    IL: CellDeserialize<'de>,
{
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, CellParserError<'de>> {
        Ok(Self {
            info: parser.unpack()?,
            init: parser
                .parse_as::<_, Option<Either<Same, Ref>>>()?
                .map(Either::into_inner),
            body: parser
                .parse_as::<Either<T, T>, Either<Same, Ref>>()?
                .into_inner(),
        })
    }
}

pub enum CommonMsgInfo {
    /// int_msg_info$0
    Internal(InternalMsgInfo),
    /// ext_in_msg_info$10
    ExternalIn(ExternalInMsgInfo),
    /// ext_out_msg_info$11
    ExternalOut(ExternalOutMsgInfo),
}

impl BitPack for CommonMsgInfo {
    fn pack<W>(&self, mut writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        match self {
            Self::Internal(msg) => writer
                // int_msg_info$0
                .pack(false)?
                .pack(msg)?,
            Self::ExternalIn(msg) => writer
                // ext_in_msg_info$10
                .pack_as::<_, NBits<2>>(0b10)?
                .pack(msg)?,
            Self::ExternalOut(msg) => writer
                // ext_out_msg_info$11
                .pack_as::<_, NBits<2>>(0b11)?
                .pack(msg)?,
        };
        Ok(())
    }
}

impl BitUnpack for CommonMsgInfo {
    fn unpack<R>(mut reader: R) -> Result<Self, R::Error>
    where
        R: BitReader,
    {
        match reader.unpack()? {
            // int_msg_info$0
            false => Ok(Self::Internal(reader.unpack()?)),
            true => match reader.unpack()? {
                // ext_in_msg_info$10
                false => Ok(Self::ExternalIn(reader.unpack()?)),
                // ext_out_msg_info$11
                true => Ok(Self::ExternalOut(reader.unpack()?)),
            },
        }
    }
}

/// int_msg_info$0 ihr_disabled:Bool bounce:Bool bounced:Bool
/// src:MsgAddressInt dest:MsgAddressInt
/// value:CurrencyCollection ihr_fee:Grams fwd_fee:Grams
/// created_lt:uint64 created_at:uint32 = CommonMsgInfo;
pub struct InternalMsgInfo {
    /// Hyper cube routing flag.
    pub ihr_disabled: bool,
    /// Message should be bounced if there are errors during processing.
    /// If message's flat bounce = 1, it calls bounceable.
    pub bounce: bool,
    /// Flag that describes, that message itself is a result of bounce.
    pub bounced: bool,
    /// Address of smart contract sender of message.
    pub src: MsgAddress,
    /// Address of smart contract destination of message.
    pub dst: MsgAddress,
    /// Structure which describes currency information including total funds transferred in message.
    pub value: CurrencyCollection,
    /// Fees for hyper routing delivery
    pub ihr_fee: BigUint,
    /// Fees for forwarding messages assigned by validators
    pub fwd_fee: BigUint,
    /// Logic time of sending message assigned by validator. Using for odering actions in smart contract.
    pub created_lt: u64,
    /// Unix time
    pub created_at: u32,
}

impl BitPack for InternalMsgInfo {
    fn pack<W>(&self, mut writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        writer
            .pack(self.ihr_disabled)?
            .pack(self.bounce)?
            .pack(self.bounced)?
            .pack(self.src)?
            .pack(self.dst)?
            .pack(&self.value)?
            .pack_as::<_, &Grams>(&self.ihr_fee)?
            .pack_as::<_, &Grams>(&self.fwd_fee)?
            .pack(self.created_lt)?
            .pack(self.created_at)?;
        Ok(())
    }
}

impl BitUnpack for InternalMsgInfo {
    fn unpack<R>(mut reader: R) -> Result<Self, R::Error>
    where
        R: BitReader,
    {
        Ok(Self {
            ihr_disabled: reader.unpack()?,
            bounce: reader.unpack()?,
            bounced: reader.unpack()?,
            src: reader.unpack()?,
            dst: reader.unpack()?,
            value: reader.unpack()?,
            ihr_fee: reader.unpack_as::<_, Grams>()?,
            fwd_fee: reader.unpack_as::<_, Grams>()?,
            created_lt: reader.unpack()?,
            created_at: reader.unpack()?,
        })
    }
}

/// ext_in_msg_info$10 src:MsgAddressExt dest:MsgAddressInt
/// import_fee:Grams = CommonMsgInfo;
pub struct ExternalInMsgInfo {
    pub src: MsgAddress,
    pub dst: MsgAddress,
    pub import_fee: BigUint,
}

impl BitPack for ExternalInMsgInfo {
    fn pack<W>(&self, mut writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        writer
            .pack(self.src)?
            .pack(self.dst)?
            .pack_as::<_, &Grams>(&self.import_fee)?;
        Ok(())
    }
}

impl BitUnpack for ExternalInMsgInfo {
    fn unpack<R>(mut reader: R) -> Result<Self, R::Error>
    where
        R: BitReader,
    {
        Ok(Self {
            src: reader.unpack()?,
            dst: reader.unpack()?,
            import_fee: reader.unpack_as::<_, Grams>()?,
        })
    }
}

/// ext_out_msg_info$11 src:MsgAddressInt dest:MsgAddressExt
/// created_lt:uint64 created_at:uint32 = CommonMsgInfo;
pub struct ExternalOutMsgInfo {
    pub src: MsgAddress,
    pub dst: MsgAddress,
    pub created_lt: u64,
    pub created_at: u32,
}

impl BitPack for ExternalOutMsgInfo {
    fn pack<W>(&self, mut writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        writer
            .pack(self.src)?
            .pack(self.dst)?
            .pack(self.created_lt)?
            .pack(self.created_at)?;
        Ok(())
    }
}

impl BitUnpack for ExternalOutMsgInfo {
    fn unpack<R>(mut reader: R) -> Result<Self, R::Error>
    where
        R: BitReader,
    {
        Ok(Self {
            src: reader.unpack()?,
            dst: reader.unpack()?,
            created_lt: reader.unpack()?,
            created_at: reader.unpack()?,
        })
    }
}

/// tick_tock$_ tick:Bool tock:Bool = TickTock;
#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone)]
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
