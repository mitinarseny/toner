use std::sync::Arc;

use lazy_static::lazy_static;
use nacl::sign::PUBLIC_KEY_LENGTH;
use tlb::{BitWriterExt, Cell, CellBuilder, CellBuilderError, CellSerialize};
use tlb_ton::BagOfCells;

use crate::{WalletOpSendMessage, WalletVersion};

lazy_static! {
    static ref WALLET_V4R2_CODE: Arc<Cell> = {
        BagOfCells::parse_base64(include_str!("./wallet_v4r2.code"))
            .unwrap()
            .single_root()
            .expect("code BoC must be single root")
            .clone()
    };
}

pub struct V4R2;

impl WalletVersion for V4R2 {
    type Data = WalletV4R2Data;
    type Message = WalletV4R2Message;

    fn code() -> Arc<Cell> {
        WALLET_V4R2_CODE.clone()
    }

    fn init_data(wallet_id: u32, pubkey: [u8; 32]) -> Self::Data {
        WalletV4R2Data {
            seqno: 0,
            wallet_id,
            pubkey,
        }
    }

    fn create_external_body(
        wallet_id: u32,
        expire_at: u32,
        seqno: u32,
        msgs: impl IntoIterator<Item = WalletOpSendMessage>,
    ) -> Self::Message {
        WalletV4R2Message {
            wallet_id,
            expire_at,
            seqno,
            op: WalletV4R2Op::Send(msgs.into_iter().collect()),
        }
    }
}

pub struct WalletV4R2Data {
    pub seqno: u32,
    pub wallet_id: u32,
    pub pubkey: [u8; PUBLIC_KEY_LENGTH],
}

impl CellSerialize for WalletV4R2Data {
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        builder
            .pack(self.seqno)?
            .pack(self.wallet_id)?
            .pack(self.pubkey)?
            // TODO: handle plugins dict
            .pack(false)?;
        Ok(())
    }
}

pub struct WalletV4R2Message {
    pub wallet_id: u32,
    pub expire_at: u32,
    pub seqno: u32,
    pub op: WalletV4R2Op,
}

impl CellSerialize for WalletV4R2Message {
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        builder
            .pack(self.wallet_id)?
            .pack(self.expire_at)?
            .pack(self.seqno)?
            .store(&self.op)?;
        Ok(())
    }
}

pub enum WalletV4R2Op {
    Send(Vec<WalletOpSendMessage>),
    // TODO: add support for plugins management
}

impl CellSerialize for WalletV4R2Op {
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        match self {
            Self::Send(msgs) => builder.pack(0u8)?.store_many(msgs)?,
        };
        Ok(())
    }
}
