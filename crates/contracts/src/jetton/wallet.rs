use bitvec::{mem::bits_of, order::Msb0, vec::BitVec};
use num_bigint::BigUint;
use tlb::{
    bits::{
        de::{BitReader, BitReaderExt, BitUnpack},
        integer::ConstU32,
        r#as::{Remainder, VarInt},
        ser::{BitPack, BitWriter, BitWriterExt},
    },
    de::{CellDeserialize, CellParser, CellParserError},
    either::Either,
    r#as::{ParseFully, Ref, Same},
    ser::{CellBuilder, CellBuilderError, CellSerialize, CellSerializeExt},
    Cell, Error,
};
use tlb_ton::MsgAddress;

/// Jetton Transfer message from [TEP-74](https://github.com/ton-blockchain/TEPs/blob/master/text/0074-jettons-standard.md#tl-b-schema)
/// ```tlb
/// transfer#0f8a7ea5 query_id:uint64 amount:(VarUInteger 16) destination:MsgAddress
/// response_destination:MsgAddress custom_payload:(Maybe ^Cell)
/// forward_ton_amount:(VarUInteger 16) forward_payload:(Either Cell ^Cell)
/// = InternalMsgBody;
/// ```
pub struct JettonTransfer<P = Cell, F = Cell> {
    pub query_id: u64,
    pub amount: BigUint,
    pub dst: MsgAddress,
    pub response_dst: MsgAddress,
    pub custom_payload: Option<P>,
    pub forward_ton_amount: BigUint,
    pub forward_payload: ForwardPayload<F>,
}

const JETTON_TRANSFER_TAG: u32 = 0x0f8a7ea5;

impl<P, F> CellSerialize for JettonTransfer<P, F>
where
    P: CellSerialize,
    F: CellSerialize,
{
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        builder
            // transfer#0f8a7ea5
            .pack(JETTON_TRANSFER_TAG)?
            // query_id:uint64
            .pack(self.query_id)?
            // amount:(VarUInteger 16)
            .pack_as::<_, &VarInt<4>>(&self.amount)?
            // destination:MsgAddress
            .pack(self.dst)?
            // response_destination:MsgAddress
            .pack(self.response_dst)?
            // custom_payload:(Maybe ^Cell)
            .store_as::<_, Option<Ref>>(self.custom_payload.as_ref())?
            // forward_ton_amount:(VarUInteger 16)
            .pack_as::<_, &VarInt<4>>(&self.forward_ton_amount)?
            // forward_payload:(Either Cell ^Cell)
            .store_as::<_, Either<(), Ref>>(
                Some(&self.forward_payload.to_cell()?)
                    // store empty cell inline
                    .filter(|cell| !cell.is_empty()),
            )?;
        Ok(())
    }
}

impl<'de, P, F> CellDeserialize<'de> for JettonTransfer<P, F>
where
    P: CellDeserialize<'de>,
    F: CellDeserialize<'de>,
{
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, CellParserError<'de>> {
        // transfer#0f8a7ea5
        parser.unpack::<ConstU32<JETTON_TRANSFER_TAG>>()?;
        Ok(Self {
            // query_id:uint64
            query_id: parser.unpack()?,
            // amount:(VarUInteger 16)
            amount: parser.unpack_as::<_, VarInt<4>>()?,
            // destination:MsgAddress
            dst: parser.unpack()?,
            // response_destination:MsgAddress
            response_dst: parser.unpack()?,
            // custom_payload:(Maybe ^Cell)
            custom_payload: parser.parse_as::<_, Option<Ref<ParseFully>>>()?,
            // forward_ton_amount:(VarUInteger 16)
            forward_ton_amount: parser.unpack_as::<_, VarInt<4>>()?,
            // forward_payload:(Either Cell ^Cell)
            forward_payload: parser
                .parse_as::<Either<ForwardPayload<F>, ForwardPayload<F>>, Either<ParseFully, Ref<ParseFully>>>()?
                .into_inner(),
        })
    }
}

pub enum ForwardPayload<T = Cell> {
    Data(T),
    Comment(ForwardPayloadComment),
}

impl<T> ForwardPayload<T> {
    const COMMENT_PREFIX: u32 = 0x00000000;
}

impl<T> CellSerialize for ForwardPayload<T>
where
    T: CellSerialize,
{
    #[inline]
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        match self {
            Self::Data(data) => builder.store(data)?,
            Self::Comment(comment) => builder.pack(Self::COMMENT_PREFIX)?.pack(comment)?,
        };
        Ok(())
    }
}

impl<'de, F> CellDeserialize<'de> for ForwardPayload<F>
where
    F: CellDeserialize<'de>,
{
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, CellParserError<'de>> {
        if parser.bits_left() >= bits_of::<u32>()
            // clone, so we don't advance original parser
            && parser.clone().unpack::<u32>()? == Self::COMMENT_PREFIX
        {
            // skip the prefix
            let _ = parser.unpack::<u32>()?;
            return parser.unpack().map(Self::Comment);
        }
        parser.parse().map(Self::Data)
    }
}

pub enum ForwardPayloadComment {
    Text(String),
    Binary(Vec<u8>),
}

impl ForwardPayloadComment {
    const BINARY_PREFIX: u8 = 0xff;
}

impl BitPack for ForwardPayloadComment {
    #[inline]
    fn pack<W>(&self, mut writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        match self {
            Self::Text(comment) => writer.pack(comment)?,
            Self::Binary(comment) => writer.pack(Self::BINARY_PREFIX)?.pack(comment)?,
        };
        Ok(())
    }
}

impl BitUnpack for ForwardPayloadComment {
    #[inline]
    fn unpack<R>(mut reader: R) -> Result<Self, R::Error>
    where
        R: BitReader,
    {
        let mut buf = BitVec::<u8, Msb0>::new();
        let mut r = reader.tee(&mut buf);
        if r.bits_left() >= bits_of::<u8>() && r.unpack::<u8>()? == Self::BINARY_PREFIX {
            return r.unpack_as::<_, Remainder>().map(Self::Binary);
        }
        reader = r.into_inner();
        let mut r = buf.join(reader);
        r.unpack_as::<_, Remainder>()
            .map(Self::Text)
            .map_err(Error::custom)
    }
}

/// Jetton Transfer Notification message from[TEP-74](https://github.com/ton-blockchain/TEPs/blob/master/text/0074-jettons-standard.md#tl-b-schema)
/// ```tlb
/// transfer_notification#7362d09c query_id:uint64 amount:(VarUInteger 16)
/// sender:MsgAddress forward_payload:(Either Cell ^Cell)
/// = InternalMsgBody;
/// ```
pub struct JettonTransferNotification<P = Cell> {
    pub query_id: u64,
    pub amount: BigUint,
    pub sender: MsgAddress,
    pub forward_payload: ForwardPayload<P>,
}

const JETTON_TRANSFER_NOTIFICATION_TAG: u32 = 0x7362d09c;

impl<P> CellSerialize for JettonTransferNotification<P>
where
    P: CellSerialize,
{
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        builder
            .pack(JETTON_TRANSFER_NOTIFICATION_TAG)?
            .pack(self.query_id)?
            .pack_as::<_, &VarInt<4>>(&self.amount)?
            .pack(self.sender)?
            .store_as::<Either<(), _>, Either<Same, Ref>>(Either::Right(&self.forward_payload))?;
        Ok(())
    }
}

impl<'de, P> CellDeserialize<'de> for JettonTransferNotification<P>
where
    P: CellDeserialize<'de>,
{
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, CellParserError<'de>> {
        parser.unpack::<ConstU32<JETTON_TRANSFER_NOTIFICATION_TAG>>()?;
        Ok(Self {
            query_id: parser.unpack()?,
            amount: parser.unpack_as::<_, VarInt<4>>()?,
            sender: parser.unpack()?,
            forward_payload: parser
                .parse_as::<Either<ForwardPayload<P>, ForwardPayload<P>>, Either<Same, Ref<ParseFully>>>()?
                .into_inner(),
        })
    }
}

/// Jetton Burn message from [TEP-74](https://github.com/ton-blockchain/TEPs/blob/master/text/0074-jettons-standard.md#tl-b-schema)
/// ```tlb
/// burn#595f07bc query_id:uint64 amount:(VarUInteger 16)
/// response_destination:MsgAddress custom_payload:(Maybe ^Cell)
/// = InternalMsgBody;
/// ```
pub struct JettonBurn<P = Cell> {
    pub query_id: u64,
    pub amount: BigUint,
    pub response_dst: MsgAddress,
    pub custom_payload: Option<P>,
}

const JETTON_BURN_TAG: u32 = 0x595f07bc;

impl<P> CellSerialize for JettonBurn<P>
where
    P: CellSerialize,
{
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        builder
            .pack(JETTON_BURN_TAG)?
            .pack_as::<_, &VarInt<4>>(&self.amount)?
            .pack(self.response_dst)?
            .store_as::<_, Option<Ref>>(self.custom_payload.as_ref())?;
        Ok(())
    }
}

impl<'de, P> CellDeserialize<'de> for JettonBurn<P>
where
    P: CellDeserialize<'de>,
{
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, CellParserError<'de>> {
        parser.unpack::<ConstU32<JETTON_BURN_TAG>>()?;
        Ok(Self {
            query_id: parser.unpack()?,
            amount: parser.unpack_as::<_, VarInt<4>>()?,
            response_dst: parser.unpack()?,
            custom_payload: parser.parse_as::<_, Option<Ref<ParseFully>>>()?,
        })
    }
}
