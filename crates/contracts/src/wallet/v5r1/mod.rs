use std::sync::Arc;

use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use nacl::sign::PUBLIC_KEY_LENGTH;
use tlb::{
    bits::{de::BitReaderExt, ser::BitWriterExt},
    de::{CellDeserialize, CellParser, CellParserError},
    r#as::{Data, NoArgs},
    ser::{CellBuilder, CellBuilderError, CellSerialize},
    Cell, Error, ResultExt,
};
use tlb_ton::{
    action::{OutAction, SendMsgAction},
    boc::BagOfCells,
    hashmap::HashmapE,
    list::List,
    MsgAddress, UnixTimestamp,
};

use super::WalletVersion;

lazy_static! {
    static ref WALLET_V5R1_CODE_CELL: Arc<Cell> = {
        BagOfCells::parse_base64(include_str!("./wallet_v5r1.code"))
            .unwrap()
            .single_root()
            .expect("code BoC must be single root")
            .clone()
    };
}

/// Wallet [v5r1](https://github.com/ton-blockchain/wallet-contract-v5/blob/main/Specification.md).
pub struct V5R1;

impl WalletVersion for V5R1 {
    type Data = WalletV5R1Data;
    type SignBody = WalletV5RSignBody;
    type ExternalMsgBody = WalletV5R1MsgBody;

    #[inline]
    fn code() -> Arc<Cell> {
        WALLET_V5R1_CODE_CELL.clone()
    }

    #[inline]
    fn init_data(wallet_id: u32, pubkey: [u8; nacl::sign::PUBLIC_KEY_LENGTH]) -> Self::Data {
        WalletV5R1Data {
            is_signature_allowed: true,
            seqno: 0,
            wallet_id,
            pubkey,
            extensions: HashmapE::Empty,
        }
    }

    #[inline]
    fn create_sign_body(
        wallet_id: u32,
        valid_until: DateTime<Utc>,
        msg_seqno: u32,
        msgs: impl IntoIterator<Item = SendMsgAction>,
    ) -> Self::SignBody {
        WalletV5RSignBody {
            wallet_id,
            valid_until,
            msg_seqno,
            inner: WalletV5R1InnerRequest {
                out_actions: msgs.into_iter().map(OutAction::SendMsg).collect(),
                extended: [].into(),
            },
        }
    }

    #[inline]
    fn wrap_signed_external(body: Self::SignBody, signature: [u8; 64]) -> Self::ExternalMsgBody {
        WalletV5R1MsgBody::ExternalSigned(WalletV5R1SignedRequest { body, signature })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalletV5R1Data {
    pub is_signature_allowed: bool,
    pub seqno: u32,
    pub wallet_id: u32,
    pub pubkey: [u8; PUBLIC_KEY_LENGTH],
    pub extensions: HashmapE<bool>,
}

impl CellSerialize for WalletV5R1Data {
    #[inline]
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        builder
            .pack(self.is_signature_allowed)?
            .pack(self.seqno)?
            .pack(self.wallet_id)?
            .pack(self.pubkey)?
            .store_as_with::<_, &HashmapE<Data<NoArgs<_>>, NoArgs<_>>>(
                &self.extensions,
                (256, (), ()),
            )?;
        Ok(())
    }
}

impl<'de> CellDeserialize<'de> for WalletV5R1Data {
    #[inline]
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, CellParserError<'de>> {
        Ok(Self {
            is_signature_allowed: parser.unpack()?,
            seqno: parser.unpack()?,
            wallet_id: parser.unpack()?,
            pubkey: parser.unpack()?,
            extensions: parser.parse_as_with::<_, HashmapE<Data<NoArgs<_>>, NoArgs<_>>>((
                258,
                (),
                (),
            ))?,
        })
    }
}

/// ```tlb
/// actions$_ out_actions:(Maybe OutList) has_other_actions:(## 1) {m:#} {n:#} other_actions:(ActionList n m) = InnerRequest;
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalletV5R1InnerRequest {
    pub out_actions: Vec<OutAction>,
    pub extended: Vec<ExtendedAction>,
}

impl CellSerialize for WalletV5R1InnerRequest {
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        builder
            .store_as::<_, Option<&List>>(
                Some(&self.out_actions).filter(|actions| !actions.is_empty()),
            )?
            .store_as::<_, Option<&List>>(Some(&self.extended).filter(|other| !other.is_empty()))?;
        Ok(())
    }
}

impl<'de> CellDeserialize<'de> for WalletV5R1InnerRequest {
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, CellParserError<'de>> {
        Ok(Self {
            out_actions: parser
                .parse_as::<_, Option<List>>()
                .context("out_actions")?
                .unwrap_or_default(),
            extended: parser
                .parse_as::<_, Option<List>>()
                .context("extended")?
                .unwrap_or_default(),
        })
    }
}

/// ```tlb
/// action_add_ext#02 addr:MsgAddressInt = ExtendedAction;
/// action_delete_ext#03 addr:MsgAddressInt = ExtendedAction;
/// action_set_signature_auth_allowed#04 allowed:(## 1) = ExtendedAction;
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtendedAction {
    /// ```tlb
    /// action_add_ext#02 addr:MsgAddressInt = ExtendedAction;
    /// ```
    AddExtension(MsgAddress),

    /// ```tlb
    /// action_delete_ext#03 addr:MsgAddressInt = ExtendedAction;
    /// ```
    DeleteExtension(MsgAddress),

    /// ```tlb
    /// action_set_signature_auth_allowed#04 allowed:(## 1) = ExtendedAction;
    /// ```
    SetSignatureAuthAllowed(bool),
}

impl ExtendedAction {
    const ADD_EXTENSION_PREFIX: u8 = 0x02;
    const DELETE_EXTENSION_PREFIX: u8 = 0x03;
    const SET_SIGNATURE_AUTH_ALLOWED_PREFIX: u8 = 0x04;
}

impl CellSerialize for ExtendedAction {
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        match self {
            Self::AddExtension(addr) => builder.pack(Self::ADD_EXTENSION_PREFIX)?.pack(addr)?,
            Self::DeleteExtension(addr) => {
                builder.pack(Self::DELETE_EXTENSION_PREFIX)?.pack(addr)?
            }
            Self::SetSignatureAuthAllowed(allowed) => builder
                .pack(Self::SET_SIGNATURE_AUTH_ALLOWED_PREFIX)?
                .pack(allowed)?,
        };
        Ok(())
    }
}

impl<'de> CellDeserialize<'de> for ExtendedAction {
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, CellParserError<'de>> {
        Ok(match parser.unpack()? {
            Self::ADD_EXTENSION_PREFIX => Self::AddExtension(parser.unpack()?),
            Self::DELETE_EXTENSION_PREFIX => Self::DeleteExtension(parser.unpack()?),
            Self::SET_SIGNATURE_AUTH_ALLOWED_PREFIX => {
                Self::SetSignatureAuthAllowed(parser.unpack()?)
            }
            prefix => return Err(Error::custom(format!("unknown prefix: {prefix:#0x}"))),
        })
    }
}

/// ```tlb
/// signed_request$_             // 32 (opcode from outer)
///  wallet_id:    #            // 32
///  valid_until:  #            // 32
///  msg_seqno:    #            // 32
///  inner:        InnerRequest //
///  signature:    bits512      // 512
///= SignedRequest;             // Total: 688 .. 976 + ^Cell
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalletV5RSignBody {
    pub wallet_id: u32,
    pub valid_until: DateTime<Utc>,
    pub msg_seqno: u32,
    pub inner: WalletV5R1InnerRequest,
}

impl CellSerialize for WalletV5RSignBody {
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        builder
            .pack(self.wallet_id)?
            .pack_as::<_, UnixTimestamp>(self.valid_until)?
            .pack(self.msg_seqno)?
            .store(&self.inner)?;
        Ok(())
    }
}

impl<'de> CellDeserialize<'de> for WalletV5RSignBody {
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, CellParserError<'de>> {
        Ok(Self {
            wallet_id: parser.unpack()?,
            valid_until: parser.unpack_as::<_, UnixTimestamp>()?,
            msg_seqno: parser.unpack()?,
            inner: parser.parse()?,
        })
    }
}

/// ```tlb
/// signed_request$_             // 32 (opcode from outer)
///  wallet_id:    #            // 32
///  valid_until:  #            // 32
///  msg_seqno:    #            // 32
///  inner:        InnerRequest //
///  signature:    bits512      // 512
///= SignedRequest;             // Total: 688 .. 976 + ^Cell
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalletV5R1SignedRequest {
    pub body: WalletV5RSignBody,
    pub signature: [u8; 64],
}

impl CellSerialize for WalletV5R1SignedRequest {
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        builder.store(&self.body)?.pack(self.signature)?;
        Ok(())
    }
}

impl<'de> CellDeserialize<'de> for WalletV5R1SignedRequest {
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, CellParserError<'de>> {
        Ok(Self {
            body: parser.parse()?,
            signature: parser.unpack()?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WalletV5R1MsgBody {
    /// ```tlb
    /// internal_signed#73696e74 signed:SignedRequest = InternalMsgBody;
    /// ```
    InternalSigned(WalletV5R1SignedRequest),

    /// ```tlb
    /// internal_extension#6578746e query_id:(## 64) inner:InnerRequest = InternalMsgBody;
    /// ```
    InternalExtension(InternalExtensionWalletV5R1MsgBody),

    /// ```tlb
    /// external_signed#7369676e signed:SignedRequest = ExternalMsgBody;
    /// ```
    ExternalSigned(WalletV5R1SignedRequest),
}

impl WalletV5R1MsgBody {
    const INTERNAL_SIGNED_PREFIX: u32 = 0x73696e74;
    const INTERNAL_EXTENSION_PREFIX: u32 = 0x6578746e;
    const EXTERNAL_SIGNED_PREFIX: u32 = 0x7369676e;
}

impl CellSerialize for WalletV5R1MsgBody {
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        match self {
            Self::InternalSigned(msg) => builder.pack(Self::INTERNAL_SIGNED_PREFIX)?.store(msg)?,
            Self::InternalExtension(msg) => {
                builder.pack(Self::INTERNAL_EXTENSION_PREFIX)?.store(msg)?
            }
            Self::ExternalSigned(msg) => builder.pack(Self::EXTERNAL_SIGNED_PREFIX)?.store(msg)?,
        };
        Ok(())
    }
}

impl<'de> CellDeserialize<'de> for WalletV5R1MsgBody {
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, CellParserError<'de>> {
        Ok(match parser.unpack()? {
            Self::INTERNAL_SIGNED_PREFIX => {
                Self::InternalSigned(parser.parse().context("internal_signed")?)
            }
            Self::INTERNAL_EXTENSION_PREFIX => {
                Self::InternalExtension(parser.parse().context("internal_extension")?)
            }
            Self::EXTERNAL_SIGNED_PREFIX => {
                Self::ExternalSigned(parser.parse().context("external_signed")?)
            }
            prefix => return Err(Error::custom(format!("unknown prefix: {prefix:#0x}"))),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InternalExtensionWalletV5R1MsgBody {
    query_id: u64,
    inner: WalletV5R1InnerRequest,
}

impl CellSerialize for InternalExtensionWalletV5R1MsgBody {
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        builder.pack(self.query_id)?.store(&self.inner)?;
        Ok(())
    }
}

impl<'de> CellDeserialize<'de> for InternalExtensionWalletV5R1MsgBody {
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, CellParserError<'de>> {
        Ok(Self {
            query_id: parser.unpack()?,
            inner: parser.parse()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use tlb::bits::{de::unpack_fully, ser::pack_with};
    use tlb_ton::boc::{BagOfCellsArgs, BoC};

    use super::*;

    #[test]
    fn check_code() {
        let packed = pack_with(
            BoC::from_root(WALLET_V5R1_CODE_CELL.clone()),
            BagOfCellsArgs {
                has_idx: false,
                has_crc32c: true,
            },
        )
        .unwrap();

        let unpacked: BoC = unpack_fully(packed).unwrap();

        let got: Cell = unpacked.single_root().unwrap().parse_fully().unwrap();
        assert_eq!(&got, WALLET_V5R1_CODE_CELL.as_ref());
    }
}
