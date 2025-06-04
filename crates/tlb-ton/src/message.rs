//! Collection of typs related to [Message](https://docs.ton.org/develop/data-formats/msg-tlb#message-tl-b)
use chrono::{DateTime, Utc};
use num_bigint::BigUint;
use tlb::{
    Cell, Context,
    r#as::{DefaultOnNone, EitherInlineOrRef, hashmap::HashmapE},
    bits::{
        r#as::NBits,
        de::{BitReader, BitReaderExt, BitUnpack},
        ser::{BitPack, BitWriter, BitWriterExt},
    },
    de::{CellDeserialize, CellParser, CellParserError},
    ser::{CellBuilder, CellBuilderError, CellSerialize, CellSerializeExt},
};

use crate::{
    MsgAddress, UnixTimestamp,
    currency::{CurrencyCollection, ExtraCurrencyCollection, Grams},
    state_init::StateInit,
};

/// [Message](https://docs.ton.org/develop/data-formats/msg-tlb#message-tl-b)
/// ```tlb
/// message$_ {X:Type} info:CommonMsgInfo
/// init:(Maybe (Either StateInit ^StateInit))
/// body:(Either X ^X) = Message X;
/// ```
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message<T = Cell, IC = Cell, ID = Cell> {
    pub info: CommonMsgInfo,
    pub init: Option<StateInit<IC, ID>>,
    pub body: T,
}

impl<T, IC, ID> Message<T, IC, ID>
where
    T: CellSerialize,
    IC: CellSerialize,
    ID: CellSerialize,
{
    #[inline]
    pub fn with_state_init(mut self, state_init: impl Into<Option<StateInit<IC, ID>>>) -> Self {
        self.init = state_init.into();
        self
    }

    #[inline]
    pub fn normalize(&self) -> Result<Message, CellBuilderError> {
        Ok(Message {
            info: self.info.clone(),
            init: self.init.as_ref().map(StateInit::normalize).transpose()?,
            body: self.body.to_cell()?,
        })
    }
}

impl Message<()> {
    /// Simple native transfer message
    #[inline]
    pub const fn transfer(dst: MsgAddress, grams: BigUint, bounce: bool) -> Self {
        Self {
            info: CommonMsgInfo::transfer(dst, grams, bounce),
            init: None,
            body: (),
        }
    }
}

impl<T, IC, ID> CellSerialize for Message<T, IC, ID>
where
    T: CellSerialize,
    IC: CellSerialize,
    ID: CellSerialize,
{
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        builder
            // info:CommonMsgInfo
            .store(&self.info)?
            // init:(Maybe (Either StateInit ^StateInit))
            .store_as::<_, &Option<EitherInlineOrRef>>(&self.init)?
            // body:(Either X ^X)
            .store_as::<_, EitherInlineOrRef>(&self.body)?;
        Ok(())
    }
}

impl<'de, T, IC, ID> CellDeserialize<'de> for Message<T, IC, ID>
where
    T: CellDeserialize<'de>,
    IC: CellDeserialize<'de>,
    ID: CellDeserialize<'de>,
{
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, CellParserError<'de>> {
        Ok(Self {
            // info:CommonMsgInfo
            info: parser.parse().context("info")?,
            // init:(Maybe (Either StateInit ^StateInit))
            init: parser
                .parse_as::<_, Option<EitherInlineOrRef>>()
                .context("init")?,
            // body:(Either X ^X)
            body: parser.parse_as::<_, EitherInlineOrRef>().context("body")?,
        })
    }
}

/// `info` field for [`Message`]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommonMsgInfo {
    /// ```tlb
    /// int_msg_info$0
    /// ```
    Internal(InternalMsgInfo),

    /// ```tlb
    /// ext_in_msg_info$10
    /// ```
    ExternalIn(ExternalInMsgInfo),

    /// ```tlb
    /// ext_out_msg_info$11
    /// ```
    ExternalOut(ExternalOutMsgInfo),
}

impl CommonMsgInfo {
    #[inline]
    pub const fn transfer(dst: MsgAddress, grams: BigUint, bounce: bool) -> Self {
        Self::Internal(InternalMsgInfo::transfer(dst, grams, bounce))
    }
}

impl CellSerialize for CommonMsgInfo {
    #[inline]
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        match self {
            Self::Internal(msg) => builder
                // int_msg_info$0
                .pack(false)?
                .store(msg)?,
            Self::ExternalIn(msg) => builder
                // ext_in_msg_info$10
                .pack_as::<_, NBits<2>>(0b10)?
                .pack(msg)?,
            Self::ExternalOut(msg) => builder
                // ext_out_msg_info$11
                .pack_as::<_, NBits<2>>(0b11)?
                .pack(msg)?,
        };
        Ok(())
    }
}

impl<'de> CellDeserialize<'de> for CommonMsgInfo {
    #[inline]
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, CellParserError<'de>> {
        match parser.unpack()? {
            // int_msg_info$0
            false => Ok(Self::Internal(parser.parse().context("int_msg_info")?)),
            true => match parser.unpack()? {
                // ext_in_msg_info$10
                false => Ok(Self::ExternalIn(
                    parser.unpack().context("ext_in_msg_info")?,
                )),
                // ext_out_msg_info$11
                true => Ok(Self::ExternalOut(
                    parser.unpack().context("ext_out_msg_info")?,
                )),
            },
        }
    }
}

/// [`int_msg_info$0`](https://docs.ton.org/develop/data-formats/msg-tlb#int_msg_info0)
/// ```tlb
/// int_msg_info$0 ihr_disabled:Bool bounce:Bool bounced:Bool
/// src:MsgAddressInt dest:MsgAddressInt
/// value:CurrencyCollection ihr_fee:Grams fwd_fee:Grams
/// created_lt:uint64 created_at:uint32 = CommonMsgInfo;
/// ```
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq)]
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
    #[cfg_attr(
        feature = "arbitrary",
        arbitrary(with = UnixTimestamp::arbitrary_option)
    )]
    pub created_at: Option<DateTime<Utc>>,
}

impl InternalMsgInfo {
    #[inline]
    pub const fn transfer(dst: MsgAddress, grams: BigUint, bounce: bool) -> Self {
        InternalMsgInfo {
            ihr_disabled: true,
            bounce,
            bounced: false,
            src: MsgAddress::NULL,
            dst,
            value: CurrencyCollection {
                grams,
                other: ExtraCurrencyCollection(HashmapE::Empty),
            },
            ihr_fee: BigUint::ZERO,
            fwd_fee: BigUint::ZERO,
            created_lt: 0,
            created_at: None,
        }
    }
}

impl CellSerialize for InternalMsgInfo {
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        builder
            .pack(self.ihr_disabled)?
            .pack(self.bounce)?
            .pack(self.bounced)?
            .pack(self.src)?
            .pack(self.dst)?
            .store(&self.value)?
            .pack_as::<_, &Grams>(&self.ihr_fee)?
            .pack_as::<_, &Grams>(&self.fwd_fee)?
            .pack(self.created_lt)?
            .pack_as::<_, DefaultOnNone<UnixTimestamp>>(self.created_at)?;
        Ok(())
    }
}

impl<'de> CellDeserialize<'de> for InternalMsgInfo {
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, CellParserError<'de>> {
        Ok(Self {
            ihr_disabled: parser.unpack()?,
            bounce: parser.unpack()?,
            bounced: parser.unpack()?,
            src: parser.unpack().context("src")?,
            dst: parser.unpack().context("dst")?,
            value: parser.parse().context("value")?,
            ihr_fee: parser.unpack_as::<_, Grams>()?,
            fwd_fee: parser.unpack_as::<_, Grams>()?,
            created_lt: parser.unpack()?,
            created_at: Some(parser.unpack_as::<_, UnixTimestamp>()?)
                .filter(|dt| *dt != DateTime::UNIX_EPOCH),
        })
    }
}

/// [`ext_in_msg_info$10`](https://docs.ton.org/develop/data-formats/msg-tlb#ext_in_msg_info10)
/// ```tlb
/// ext_in_msg_info$10 src:MsgAddressExt dest:MsgAddressInt
/// import_fee:Grams = CommonMsgInfo;
/// ```
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq)]
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

impl<'de> BitUnpack<'de> for ExternalInMsgInfo {
    fn unpack<R>(mut reader: R) -> Result<Self, R::Error>
    where
        R: BitReader<'de>,
    {
        Ok(Self {
            src: reader.unpack()?,
            dst: reader.unpack()?,
            import_fee: reader.unpack_as::<_, Grams>()?,
        })
    }
}

/// [`ext_out_msg_info$11`](https://docs.ton.org/develop/data-formats/msg-tlb#ext_out_msg_info11)
/// ```tlb
/// ext_out_msg_info$11 src:MsgAddressInt dest:MsgAddressExt
/// created_lt:uint64 created_at:uint32 = CommonMsgInfo;
/// ```
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalOutMsgInfo {
    pub src: MsgAddress,
    pub dst: MsgAddress,
    pub created_lt: u64,
    pub created_at: DateTime<Utc>,
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
            .pack_as::<_, UnixTimestamp>(self.created_at)?;
        Ok(())
    }
}

impl<'de> BitUnpack<'de> for ExternalOutMsgInfo {
    fn unpack<R>(mut reader: R) -> Result<Self, R::Error>
    where
        R: BitReader<'de>,
    {
        Ok(Self {
            src: reader.unpack()?,
            dst: reader.unpack()?,
            created_lt: reader.unpack()?,
            created_at: reader.unpack_as::<_, UnixTimestamp>()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use tlb::ser::CellSerializeExt;

    use super::*;

    #[test]
    fn message_serde() {
        let msg = Message::<(), (), ()> {
            info: CommonMsgInfo::Internal(InternalMsgInfo {
                ihr_disabled: true,
                bounce: true,
                bounced: false,
                src: MsgAddress::NULL,
                dst: MsgAddress::NULL,
                value: Default::default(),
                ihr_fee: BigUint::ZERO,
                fwd_fee: BigUint::ZERO,
                created_lt: 0,
                created_at: None,
            }),
            init: None,
            body: (),
        };

        let cell = msg.to_cell().unwrap();
        let got: Message<(), (), ()> = cell.parse_fully().unwrap();

        assert_eq!(got, msg);
    }

    #[test]
    fn internal_msg_info_serde() {
        let info = CommonMsgInfo::Internal(InternalMsgInfo {
            ihr_disabled: true,
            bounce: true,
            bounced: false,
            src: MsgAddress::NULL,
            dst: MsgAddress::NULL,
            value: Default::default(),
            ihr_fee: BigUint::ZERO,
            fwd_fee: BigUint::ZERO,
            created_lt: 0,
            created_at: None,
        });

        let cell = info.to_cell().unwrap();
        let got: CommonMsgInfo = cell.parse_fully().unwrap();

        assert_eq!(got, info);
    }

    #[test]
    fn external_in_msg_info_serde() {
        let info = CommonMsgInfo::ExternalIn(ExternalInMsgInfo {
            src: MsgAddress::NULL,
            dst: MsgAddress::NULL,
            import_fee: BigUint::ZERO,
        });

        let cell = info.to_cell().unwrap();
        let got: CommonMsgInfo = cell.parse_fully().unwrap();

        assert_eq!(got, info);
    }

    #[test]
    fn external_out_msg_info_serde() {
        let info = CommonMsgInfo::ExternalOut(ExternalOutMsgInfo {
            src: MsgAddress::NULL,
            dst: MsgAddress::NULL,
            created_lt: 0,
            created_at: DateTime::UNIX_EPOCH,
        });

        let cell = info.to_cell().unwrap();
        let got: CommonMsgInfo = cell.parse_fully().unwrap();

        assert_eq!(got, info);
    }
}
