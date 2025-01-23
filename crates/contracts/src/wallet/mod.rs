//! TON [Wallet](https://docs.ton.org/participate/wallets/contracts)
pub mod mnemonic;
mod signer;
pub mod v4r2;
pub mod v5r1;
mod version;

pub use self::{signer::*, version::*};

use core::marker::PhantomData;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use num_bigint::BigUint;
use tlb::{
    ser::{CellBuilderError, CellSerializeExt},
    Cell,
};
use tlb_ton::{
    action::SendMsgAction,
    message::{CommonMsgInfo, ExternalInMsgInfo, Message},
    state_init::StateInit,
    MsgAddress,
};

/// Generic wallet for signing messages
///
/// ```rust
/// # use ton_contracts::wallet::{
/// #   mnemonic::Mnemonic,
/// #   KeyPair,
/// #   Wallet,
/// #   v4r2::V4R2,
/// # };
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
    keypair: KeyPair,
    _phantom: PhantomData<V>,
}

impl<V> Wallet<V>
where
    V: WalletVersion,
{
    #[inline]
    pub const fn new(address: MsgAddress, keypair: KeyPair, wallet_id: u32) -> Self {
        Self {
            address,
            wallet_id,
            keypair,
            _phantom: PhantomData,
        }
    }

    /// Derive wallet from its workchain, keypair and id
    #[inline]
    pub fn derive(
        workchain_id: i32,
        keypair: KeyPair,
        wallet_id: u32,
    ) -> Result<Self, CellBuilderError> {
        Ok(Self::new(
            MsgAddress::derive(workchain_id, V::state_init(wallet_id, keypair.public_key))?,
            keypair,
            wallet_id,
        ))
    }

    /// Shortcut for [`Wallet::derive()`] with default workchain and wallet id
    #[inline]
    pub fn derive_default(keypair: KeyPair) -> Result<Self, CellBuilderError> {
        Self::derive(0, keypair, V::DEFAULT_WALLET_ID)
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
    pub const fn public_key(&self) -> &[u8; PUBLIC_KEY_LENGTH] {
        &self.keypair.public_key
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

    #[inline]
    pub fn sign(&self, msg: impl AsRef<[u8]>) -> anyhow::Result<[u8; 64]> {
        self.keypair.sign(msg)
    }

    /// Shortcut to [create](Wallet::create_sign_body),
    /// [sign](Wallet::sign_body) and [wrap](Wallet::wrap_external_msg) external
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
    /// #   KeyPair,
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

    /// Sign body from [`.create_sign_body()`](Wallet::create_sign_body)
    /// using this wallet's private key
    #[inline]
    pub fn sign_body(&self, msg: &V::SignBody) -> anyhow::Result<[u8; 64]> {
        self.sign(msg.to_cell()?.hash())
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
                dst: self.address(),
                import_fee: BigUint::ZERO,
            }),
            init: state_init.then(|| self.state_init()),
            body,
        }
    }

    #[inline]
    pub fn state_init(&self) -> StateInit<Arc<Cell>, V::Data> {
        V::state_init(self.wallet_id(), *self.public_key())
    }
}
