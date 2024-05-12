pub mod mnemonic;
pub mod v4r2;

use std::{marker::PhantomData, sync::Arc};

use anyhow::anyhow;
use bitvec::view::AsBits;
use chrono::{DateTime, Utc};
use nacl::sign::{signature, Keypair};
use tlb::{
    BitWriterExt, Cell, CellBuilder, CellBuilderError, CellSerialize, CellSerializeExt, Ref,
};
use tlb_ton::{ExternalInMsgInfo, Message, MsgAddress, StateInit};

pub struct Wallet<V> {
    pub address: MsgAddress,
    pub wallet_id: u32,
    pub key_pair: Keypair,
    _phantom: PhantomData<V>,
}

impl<V> Wallet<V>
where
    V: WalletVersion,
{
    pub fn derive(workchain_id: i32, key_pair: Keypair, wallet_id: u32) -> anyhow::Result<Self> {
        let state_init_hash = StateInit::<_, _, ()> {
            code: Some(V::code()),
            data: Some(V::init_data(wallet_id, key_pair.pkey)),
            ..Default::default()
        }
        .to_cell()?
        .hash();
        Ok(Self {
            address: MsgAddress {
                workchain_id,
                address: state_init_hash,
            },
            wallet_id,
            key_pair,
            _phantom: PhantomData,
        })
    }

    pub fn derive_default(key_pair: Keypair) -> anyhow::Result<Self> {
        Self::derive(0, key_pair, 0)
    }

    pub fn create_external_message(
        &self,
        expire_at: DateTime<Utc>,
        seqno: u32,
        msgs: impl IntoIterator<Item = WalletOpSendMessage>,
        state_init: bool,
    ) -> anyhow::Result<Message<SignedMessage, Arc<Cell>, V::Data, ()>> {
        let body = self.create_external_body(expire_at, seqno, msgs);
        let signed = self.sign_body(body)?;
        let wrapped = self.wrap_signed(signed, state_init);
        Ok(wrapped)
    }

    pub fn create_external_body(
        &self,
        expire_at: DateTime<Utc>,
        seqno: u32,
        msgs: impl IntoIterator<Item = WalletOpSendMessage>,
    ) -> V::MessageBody {
        V::create_external_body(self.wallet_id, expire_at, seqno, msgs)
    }

    pub fn sign_body(&self, msg: V::MessageBody) -> anyhow::Result<SignedMessage> {
        let msg = msg.to_cell()?;
        Ok(SignedMessage {
            sig: signature(msg.hash().as_slice(), self.key_pair.skey.as_slice())
                .map_err(|e| anyhow!("{}", e.message))?,
            msg,
        })
    }

    pub fn wrap_signed(
        &self,
        msg: SignedMessage,
        state_init: bool,
    ) -> Message<SignedMessage, Arc<Cell>, V::Data, ()> {
        Message {
            info: tlb_ton::CommonMsgInfo::ExternalIn(ExternalInMsgInfo {
                src: MsgAddress::NULL,
                dst: self.address,
                import_fee: 0u8.into(),
            }),
            init: state_init.then(|| StateInit::<_, _, ()> {
                code: Some(V::code()),
                data: Some(V::init_data(self.wallet_id, self.key_pair.pkey)),
                ..Default::default()
            }),
            body: msg,
        }
    }
}

pub struct SignedMessage<T = Cell> {
    pub sig: Vec<u8>,
    pub msg: T,
}

impl<T> CellSerialize for SignedMessage<T>
where
    T: CellSerialize,
{
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        builder.pack(self.sig.as_bits())?.store(&self.msg)?;
        Ok(())
    }
}

pub trait WalletVersion {
    type Data: CellSerialize;
    type MessageBody: CellSerialize;

    fn code() -> Arc<Cell>;

    fn init_data(wallet_id: u32, pubkey: [u8; 32]) -> Self::Data;

    fn create_external_body(
        wallet_id: u32,
        expire_at: DateTime<Utc>,
        seqno: u32,
        msgs: impl IntoIterator<Item = WalletOpSendMessage>,
    ) -> Self::MessageBody;
}

pub struct WalletOpSendMessage<T = Cell, IC = Cell, ID = Cell, IL = Cell> {
    /// See https://docs.ton.org/develop/func/stdlib#send_raw_message
    pub mode: u8,
    pub message: Message<T, IC, ID, IL>,
}

impl<T, IC, ID, IL> CellSerialize for WalletOpSendMessage<T, IC, ID, IL>
where
    T: CellSerialize,
    IC: CellSerialize,
    ID: CellSerialize,
    IL: CellSerialize,
{
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        builder.pack(self.mode)?.store_as::<_, Ref>(&self.message)?;
        Ok(())
    }
}
