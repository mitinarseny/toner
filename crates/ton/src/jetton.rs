use num_bigint::BigUint;
use tlb::{
    BitReaderExt, BitWriterExt, Cell, CellBuilder, CellBuilderError, CellDeserialize, CellParser,
    CellParserError, CellSerialize, ConstU32, Either, Ref, Same, VarUint,
};

use crate::MsgAddress;

/// transfer#0f8a7ea5 query_id:uint64 amount:(VarUInteger 16) destination:MsgAddress
/// response_destination:MsgAddress custom_payload:(Maybe ^Cell)
/// forward_ton_amount:(VarUInteger 16) forward_payload:(Either Cell ^Cell)
/// = InternalMsgBody;
pub struct JettonTransfer<P = Cell, F = Cell> {
    pub query_id: u64,
    pub amount: BigUint,
    pub dst: MsgAddress,
    pub response_dst: MsgAddress,
    pub custom_payload: Option<P>,
    pub forward_ton_amount: BigUint,
    pub forward_payload: F,
}

const JETTON_TRANSFER_TAG: u32 = 0x0f8a7ea5;

impl<P, F> CellSerialize for JettonTransfer<P, F>
where
    P: CellSerialize,
    F: CellSerialize,
{
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        builder
            .pack(JETTON_TRANSFER_TAG)?
            .pack(self.query_id)?
            .pack_as::<_, &VarUint<4>>(&self.amount)?
            .pack(self.dst)?
            .pack(self.response_dst)?
            .store_as::<_, Option<Ref>>(self.custom_payload.as_ref())?
            .pack_as::<_, &VarUint<4>>(&self.forward_ton_amount)?
            .store_as::<Either<(), _>, Either<Same, Ref>>(Either::Right(&self.forward_payload))?;
        Ok(())
    }
}

impl<'de, P, F> CellDeserialize<'de> for JettonTransfer<P, F>
where
    P: CellDeserialize<'de>,
    F: CellDeserialize<'de>,
{
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, CellParserError<'de>> {
        parser.unpack::<ConstU32<JETTON_TRANSFER_TAG>>()?;
        Ok(Self {
            query_id: parser.unpack()?,
            amount: parser.unpack_as::<_, VarUint<4>>()?,
            dst: parser.unpack()?,
            response_dst: parser.unpack()?,
            custom_payload: parser.parse_as::<_, Option<Ref>>()?,
            forward_ton_amount: parser.unpack_as::<_, VarUint<4>>()?,
            forward_payload: parser
                .parse_as::<Either<F, F>, Either<Same, Ref>>()?
                .into_inner(),
        })
    }
}

/// transfer_notification#7362d09c query_id:uint64 amount:(VarUInteger 16)
/// sender:MsgAddress forward_payload:(Either Cell ^Cell)
/// = InternalMsgBody;
pub struct JettonTransferNotification<P = Cell> {
    pub query_id: u64,
    pub amount: BigUint,
    pub sender: MsgAddress,
    pub forward_payload: P,
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
            .pack_as::<_, &VarUint<4>>(&self.amount)?
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
            amount: parser.unpack_as::<_, VarUint<4>>()?,
            sender: parser.unpack()?,
            forward_payload: parser
                .parse_as::<Either<P, P>, Either<Same, Ref>>()?
                .into_inner(),
        })
    }
}

/// burn#595f07bc query_id:uint64 amount:(VarUInteger 16)
/// response_destination:MsgAddress custom_payload:(Maybe ^Cell)
/// = InternalMsgBody;
pub struct JettonBurn<P> {
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
            .pack_as::<_, &VarUint<4>>(&self.amount)?
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
            amount: parser.unpack_as::<_, VarUint<4>>()?,
            response_dst: parser.unpack()?,
            custom_payload: parser.parse_as::<_, Option<Ref>>()?,
        })
    }
}
