//! TON [Wallet](https://docs.ton.org/participate/wallets/contracts)
pub mod mnemonic;
pub mod v4r2;
pub mod v5r1;

use std::{marker::PhantomData, sync::Arc};

use anyhow::anyhow;
use chrono::{DateTime, Utc};
use nacl::sign::{signature, Keypair, PUBLIC_KEY_LENGTH};
use num_bigint::BigUint;
use tlb::{ser::CellSerialize, Cell};
use tlb_ton::{
    action::SendMsgAction,
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
    // TODO
    // pub fn derive_state(workchain_id: i32, state: )

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

    #[inline]
    pub fn sign(&self, msg: impl AsRef<[u8]>) -> anyhow::Result<[u8; 64]> {
        signature(msg.as_ref(), self.key_pair.skey.as_slice())
            .map_err(|e| anyhow!("{}", e.message))?
            .try_into()
            .map_err(|sig: Vec<_>| {
                anyhow!(
                    "got signature of a wrong size, expected 64, got: {}",
                    sig.len()
                )
            })
    }

    /// Shortcut to [create](Wallet::create_external_body),
    /// [sign](Wallet::sign_body) and [wrap](Wallet::wrap_signed) external
    /// message ready for sending to TON blockchain.
    ///
    /// ```rust
    /// # use hex_literal::hex;
    /// # use tlb::Cell;
    /// # use tlb_ton::{
    /// #   message::Message,
    /// #   currency::ONE_TON,
    /// #   action::SendMsgAction,
    /// # };
    /// # use ton_contracts::wallet::{
    /// #   mnemonic::Mnemonic,
    /// #   v5r1::V5R1,
    /// #   Wallet,
    /// # };
    /// #
    /// # let mnemonic: Mnemonic = "jewel loop vast intact snack drip fatigue lunch erode green indoor balance together scrub hen monster hour narrow banner warfare increase panel sound spell"
    /// #     .parse()
    /// #     .unwrap();
    /// # let keypair = mnemonic.generate_keypair(None).unwrap();
    /// # let wallet = Wallet::<V5R1>::derive_default(keypair).unwrap();
    /// let msg = wallet.create_external_message(
    ///     Default::default(), // DateTime::UNIX_EPOCH means no deadline
    ///     0, // seqno
    ///     [SendMsgAction {
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
    ///     ).unwrap();
    /// # let mut b = Cell::builder();
    /// # b.store(msg).unwrap();
    /// # let cell = b.into_cell();
    /// # assert_eq!(
    /// #     hex!("607b41a4b219fbc6d23f4aae5c4b85e5ceca07bc0ba732ae02a621588f0577d4"),
    ///#      cell.hash(),
    /// # );
    /// ```
    #[inline]
    pub fn create_external_message(
        &self,
        expire_at: DateTime<Utc>,
        seqno: u32,
        msgs: impl IntoIterator<Item = SendMsgAction>,
        state_init: bool,
    ) -> anyhow::Result<Message<V::ExternalMsgBody, Arc<Cell>, V::Data>> {
        let sign_body = self.create_sign_body(expire_at, seqno, msgs);
        let signature = self.sign_body(&sign_body)?;
        let body = V::wrap_signed_external(sign_body, signature);
        let wrapped = self.wrap_external_msg(body, state_init);
        Ok(wrapped)
    }

    /// Create external body for this wallet.
    #[inline]
    pub fn create_sign_body(
        &self,
        expire_at: DateTime<Utc>,
        seqno: u32,
        msgs: impl IntoIterator<Item = SendMsgAction>,
    ) -> V::SignBody {
        V::create_sign_body(self.wallet_id, expire_at, seqno, msgs)
    }

    /// Sign body from [`.create_external_body()`](Wallet::create_external_body)
    /// using this wallet's private key
    pub fn sign_body(&self, msg: &V::SignBody) -> anyhow::Result<[u8; 64]> {
        let mut b = Cell::builder();
        b.store(msg)?;
        self.sign(b.into_cell().hash())
    }

    /// Wrap signed body from [`.sign_body()`](Wallet::sign_body) in a message
    /// ready for sending to TON blockchain.
    #[inline]
    pub fn wrap_external_msg(
        &self,
        body: V::ExternalMsgBody,
        state_init: bool,
    ) -> Message<V::ExternalMsgBody, Arc<Cell>, V::Data> {
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

/// Version of [`Wallet`]
pub trait WalletVersion {
    type Data: CellSerialize;
    type SignBody: CellSerialize;
    type ExternalMsgBody: CellSerialize;

    /// Code of the wallet for use with [`StateInit`]
    fn code() -> Arc<Cell>;

    /// Init data for use with [`StateInit`]
    fn init_data(wallet_id: u32, pubkey: [u8; PUBLIC_KEY_LENGTH]) -> Self::Data;

    /// Creates body for further signing with
    /// [`.wrap_signed_external()`](WalletVersion::wrap_signed_external)
    fn create_sign_body(
        wallet_id: u32,
        expire_at: DateTime<Utc>,
        seqno: u32,
        msgs: impl IntoIterator<Item = SendMsgAction>,
    ) -> Self::SignBody;

    /// Wraps signed body into external [`Message::body`]
    fn wrap_signed_external(body: Self::SignBody, signature: [u8; 64]) -> Self::ExternalMsgBody;
}
