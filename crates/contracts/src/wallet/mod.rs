//! TON [Wallet](https://docs.ton.org/participate/wallets/contracts)
pub mod mnemonic;
pub mod v4r2;

use std::{marker::PhantomData, sync::Arc};

use anyhow::anyhow;
use chrono::{DateTime, Utc};
use nacl::sign::{signature, Keypair, PUBLIC_KEY_LENGTH};
use num_bigint::BigUint;
use tlb::{
    bits::{de::BitReaderExt, ser::BitWriterExt},
    de::{CellDeserialize, CellParser, CellParserError},
    r#as::Ref,
    ser::{CellBuilder, CellBuilderError, CellSerialize, CellSerializeExt},
    Cell,
};
use tlb_ton::{
    message::{CommonMsgInfo, ExternalInMsgInfo, Message},
    state_init::StateInit,
    MsgAddress,
};

pub const DEFAULT_WALLET_ID: u32 = 0x29a9a317;

/// Generic wallet for signing messages
///
/// ```rust
/// # use ton_contracts::wallet::{mnemonic::Mnemonic, Wallet, v4r2::V4R2};
/// let mnemonic: Mnemonic = "jewel loop vast intact snack drip fatigue lunch erode green indoor balance together scrub hen monster hour narrow banner warfare increase panel sound spell"
///     .parse()
///     .unwrap();
/// let keypair = mnemonic.generate_keypair(None).unwrap();
/// let wallet = Wallet::<V4R2>::derive_default(keypair).unwrap();
///
/// assert_eq!(
///     wallet.address(),
///     "UQA7RMTgzvcyxNNLmK2HdklOvFE8_KNMa-btKZ0dPU1UsqfC".parse().unwrap(),
/// )
/// ```
pub struct Wallet<V> {
    address: MsgAddress,
    wallet_id: u32,
    key_pair: Keypair,
    _phantom: PhantomData<V>,
}

impl<V> Wallet<V>
where
    V: WalletVersion,
{
    /// Derive wallet from its workchain, keypair and id
    pub fn derive(workchain_id: i32, key_pair: Keypair, wallet_id: u32) -> anyhow::Result<Self> {
        Ok(Self {
            address: MsgAddress::derive(
                workchain_id,
                StateInit::<_, _> {
                    code: Some(V::code()),
                    data: Some(V::init_data(wallet_id, key_pair.pkey)),
                    ..Default::default()
                }
                .normalize()?,
            )?,
            wallet_id,
            key_pair,
            _phantom: PhantomData,
        })
    }

    /// Shortcut for [`Wallet::derive()`] with default workchain and wallet id
    pub fn derive_default(key_pair: Keypair) -> anyhow::Result<Self> {
        Self::derive(0, key_pair, DEFAULT_WALLET_ID)
    }

    /// Address of the wallet
    #[inline]
    pub const fn address(&self) -> MsgAddress {
        self.address
    }

    /// ID of the wallet
    #[inline]
    pub const fn wallet_id(&self) -> u32 {
        self.wallet_id
    }

    /// Shortcut to [create](Wallet::create_external_body),
    /// [sign](Wallet::sign_body) and [wrap](Wallet::wrap_signed) external
    /// message ready for sending to TON blockchain.
    ///
    /// ```rust
    /// # use tlb_ton::{message::Message, currency::ONE_TON};
    /// # use ton_contracts::wallet::{
    /// #   mnemonic::Mnemonic,
    /// #   v4r2::V4R2,
    /// #   Wallet,
    /// #   WalletOpSendMessage,
    /// # };
    /// # let mnemonic: Mnemonic = "jewel loop vast intact snack drip fatigue lunch erode green indoor balance together scrub hen monster hour narrow banner warfare increase panel sound spell"
    /// #     .parse()
    /// #     .unwrap();
    /// # let keypair = mnemonic.generate_keypair(None).unwrap();
    /// # let wallet = Wallet::<V4R2>::derive_default(keypair).unwrap();
    /// let msg = wallet.create_external_message(
    ///     Default::default(), // DateTime::UNIX_EPOCH means no deadline
    ///     0, // seqno
    ///     [WalletOpSendMessage {
    ///         mode: 3,
    ///         message: Message::<()>::transfer(
    ///             "EQAWezezpqKTbO6xjCussXDdIeJ7XxTcErjA6uD3T3r7AwTk"
    ///                 .parse()
    ///                 .unwrap(),
    ///             ONE_TON.clone(),
    ///             false,
    ///         )
    ///             .normalize()
    ///             .unwrap(),
    ///     }],
    ///     false, // do not deploy wallet
    /// );
    /// ```
    #[inline]
    pub fn create_external_message(
        &self,
        expire_at: DateTime<Utc>,
        seqno: u32,
        msgs: impl IntoIterator<Item = WalletOpSendMessage>,
        state_init: bool,
    ) -> anyhow::Result<Message<SignedBody, Arc<Cell>, V::Data>> {
        let body = self.create_external_body(expire_at, seqno, msgs);
        let signed = self.sign_body(&body)?;
        let wrapped = self.wrap_signed(signed, state_init);
        Ok(wrapped)
    }

    /// Create external body for this wallet.
    #[inline]
    pub fn create_external_body(
        &self,
        expire_at: DateTime<Utc>,
        seqno: u32,
        msgs: impl IntoIterator<Item = WalletOpSendMessage>,
    ) -> V::MessageBody {
        V::create_external_body(self.wallet_id, expire_at, seqno, msgs)
    }

    /// Sign body from [`.create_external_body()`](Wallet::create_external_body)
    /// using this wallet's private key
    pub fn sign_body(&self, msg: &V::MessageBody) -> anyhow::Result<SignedBody> {
        let msg = msg.to_cell()?;
        Ok(SignedBody {
            sig: signature(msg.hash().as_slice(), self.key_pair.skey.as_slice())
                .map_err(|e| anyhow!("{}", e.message))?
                .try_into()
                .map_err(|sig: Vec<_>| {
                    anyhow!(
                        "got signature of a wrong size, expected 64, got: {}",
                        sig.len()
                    )
                })?,
            msg,
        })
    }

    /// Wrap signed body from [`.sign_body()`](Wallet::sign_body) in a message
    /// ready for sending to TON blockchain.
    #[inline]
    pub fn wrap_signed(
        &self,
        body: SignedBody,
        state_init: bool,
    ) -> Message<SignedBody, Arc<Cell>, V::Data> {
        Message {
            info: CommonMsgInfo::ExternalIn(ExternalInMsgInfo {
                src: MsgAddress::NULL,
                dst: self.address,
                import_fee: BigUint::ZERO,
            }),
            init: state_init.then(|| StateInit::<_, _> {
                code: Some(V::code()),
                data: Some(V::init_data(self.wallet_id, self.key_pair.pkey)),
                ..Default::default()
            }),
            body,
        }
    }
}

/// Signed body retuned from [`Wallet::sign_body()`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedBody<T = Cell> {
    pub sig: [u8; 64],
    pub msg: T,
}

impl<T> CellSerialize for SignedBody<T>
where
    T: CellSerialize,
{
    #[inline]
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        builder.pack(self.sig)?.store(&self.msg)?;
        Ok(())
    }
}

impl<'de, T> CellDeserialize<'de> for SignedBody<T>
where
    T: CellDeserialize<'de>,
{
    #[inline]
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, CellParserError<'de>> {
        Ok(Self {
            sig: parser.unpack()?,
            msg: parser.parse()?,
        })
    }
}

/// Version of [`Wallet`]
pub trait WalletVersion {
    type Data: CellSerialize;
    type MessageBody: CellSerialize;

    /// Code of the wallet for use with [`StateInit`]
    fn code() -> Arc<Cell>;

    /// Init data for use with [`StateInit`]
    fn init_data(wallet_id: u32, pubkey: [u8; PUBLIC_KEY_LENGTH]) -> Self::Data;

    /// Creates external body for [`Wallet::sign_body()`]
    fn create_external_body(
        wallet_id: u32,
        expire_at: DateTime<Utc>,
        seqno: u32,
        msgs: impl IntoIterator<Item = WalletOpSendMessage>,
    ) -> Self::MessageBody;
}

/// Operation for [`Wallet`] to send message
pub struct WalletOpSendMessage<T = Cell, IC = Cell, ID = Cell> {
    /// See <https://docs.ton.org/develop/func/stdlib#send_raw_message>
    pub mode: u8,
    pub message: Message<T, IC, ID>,
}

impl<T, IC, ID> WalletOpSendMessage<T, IC, ID>
where
    T: CellSerialize,
    IC: CellSerialize,
    ID: CellSerialize,
{
    #[inline]
    pub fn normalize(&self) -> Result<WalletOpSendMessage, CellBuilderError> {
        Ok(WalletOpSendMessage {
            mode: self.mode,
            message: self.message.normalize()?,
        })
    }
}

impl<T, IC, ID> CellSerialize for WalletOpSendMessage<T, IC, ID>
where
    T: CellSerialize,
    IC: CellSerialize,
    ID: CellSerialize,
{
    #[inline]
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        builder.pack(self.mode)?.store_as::<_, Ref>(&self.message)?;
        Ok(())
    }
}
