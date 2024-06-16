use std::sync::Arc;

use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use nacl::sign::PUBLIC_KEY_LENGTH;
use num_bigint::BigUint;
use tlb::{
    bits::{de::BitReaderExt, ser::BitWriterExt},
    de::{CellDeserialize, CellParser, CellParserError},
    r#as::{NoArgs, Ref},
    ser::{CellBuilder, CellBuilderError, CellSerialize},
    Cell,
};
use tlb_ton::{
    boc::BagOfCells, currency::Grams, hashmap::HashmapE, state_init::StateInit, MsgAddress,
    UnixTimestamp,
};

use super::{WalletOpSendMessage, WalletVersion};

lazy_static! {
    static ref WALLET_V4R2_CODE_CELL: Arc<Cell> = {
        BagOfCells::parse_base64(include_str!("./wallet_v4r2.code"))
            .unwrap()
            .single_root()
            .expect("code BoC must be single root")
            .clone()
    };
}

/// Wallet [v4r2](https://github.com/ton-blockchain/wallet-contract/blob/4111fd9e3313ec17d99ca9b5b1656445b5b49d8f/README.md).
pub struct V4R2;

impl WalletVersion for V4R2 {
    type Data = WalletV4R2Data;
    type MessageBody = WalletV4R2Message;

    fn code() -> Arc<Cell> {
        WALLET_V4R2_CODE_CELL.clone()
    }

    fn init_data(wallet_id: u32, pubkey: [u8; PUBLIC_KEY_LENGTH]) -> Self::Data {
        WalletV4R2Data {
            seqno: 0,
            wallet_id,
            pubkey,
            plugins: HashmapE::Empty,
        }
    }

    fn create_external_body(
        wallet_id: u32,
        expire_at: DateTime<Utc>,
        seqno: u32,
        msgs: impl IntoIterator<Item = WalletOpSendMessage>,
    ) -> Self::MessageBody {
        WalletV4R2Message {
            wallet_id,
            expire_at,
            seqno,
            op: WalletV4R2Op::Send(msgs.into_iter().collect()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalletV4R2Data {
    pub seqno: u32,
    pub wallet_id: u32,
    pub pubkey: [u8; PUBLIC_KEY_LENGTH],
    /// plugin address -> ()
    pub plugins: HashmapE<()>,
}

impl CellSerialize for WalletV4R2Data {
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        builder
            .pack(self.seqno)?
            .pack(self.wallet_id)?
            .pack(self.pubkey)?
            .store_as_with::<_, &HashmapE<NoArgs<_>, NoArgs<_>>>(
                &self.plugins,
                (8 + 256, (), ()),
            )?;
        Ok(())
    }
}

impl<'de> CellDeserialize<'de> for WalletV4R2Data {
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, CellParserError<'de>> {
        let d = Self {
            seqno: parser.unpack()?,
            wallet_id: parser.unpack()?,
            pubkey: parser.unpack()?,
            plugins: parser.parse_as_with::<_, HashmapE<NoArgs<_>, NoArgs<_>>>((
                8 + 256,
                (),
                (),
            ))?,
        };
        Ok(d)
    }
}

pub struct WalletV4R2Message {
    pub wallet_id: u32,
    pub expire_at: DateTime<Utc>,
    pub seqno: u32,
    pub op: WalletV4R2Op,
}

impl CellSerialize for WalletV4R2Message {
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        builder
            .pack(self.wallet_id)?
            .pack_as::<_, UnixTimestamp>(self.expire_at)?
            .pack(self.seqno)?
            .store(&self.op)?;
        Ok(())
    }
}

pub enum WalletV4R2Op {
    Send(Vec<WalletOpSendMessage>),
    DeployAndInstall(WalletV4R2OpDeployAndInstallPlugin),
    Install(WalletV4R2OpPlugin),
    Remove(WalletV4R2OpPlugin),
}

impl CellSerialize for WalletV4R2Op {
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        match self {
            Self::Send(msgs) => builder.pack(0u8)?.store_many(msgs)?,
            Self::DeployAndInstall(msg) => builder.pack(1u8)?.store(msg)?,
            Self::Install(msg) => builder.pack(2u8)?.store(msg)?,
            Self::Remove(msg) => builder.pack(3u8)?.store(msg)?,
        };
        Ok(())
    }
}

pub struct WalletV4R2OpDeployAndInstallPlugin<T = Cell, IC = Cell, ID = Cell> {
    pub plugin_workchain: i8,
    pub plugin_balance: BigUint,
    pub state_init: StateInit<IC, ID>,
    pub body: T,
}

impl<T, IC, ID> CellSerialize for WalletV4R2OpDeployAndInstallPlugin<T, IC, ID>
where
    T: CellSerialize,
    IC: CellSerialize,
    ID: CellSerialize,
{
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        builder
            .pack(self.plugin_workchain)?
            .pack_as::<_, &Grams>(&self.plugin_balance)?
            .store_as::<_, Ref>(&self.state_init)?
            .store_as::<_, Ref>(&self.body)?;
        Ok(())
    }
}

impl<'de, T, IC, ID> CellDeserialize<'de> for WalletV4R2OpDeployAndInstallPlugin<T, IC, ID>
where
    T: CellDeserialize<'de>,
    IC: CellDeserialize<'de>,
    ID: CellDeserialize<'de>,
{
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, CellParserError<'de>> {
        Ok(Self {
            plugin_workchain: parser.unpack()?,
            plugin_balance: parser.unpack_as::<_, Grams>()?,
            state_init: parser.parse_as::<_, Ref>()?,
            body: parser.parse_as::<_, Ref>()?,
        })
    }
}

pub struct WalletV4R2OpPlugin {
    pub plugin_address: MsgAddress,
    pub amount: BigUint,
    pub query_id: u64,
}

impl CellSerialize for WalletV4R2OpPlugin {
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        builder
            .pack(self.plugin_address.workchain_id as i8)?
            .pack(self.plugin_address.address)?
            .pack_as::<_, &Grams>(&self.amount)?
            .pack(self.query_id)?;
        Ok(())
    }
}

impl<'de> CellDeserialize<'de> for WalletV4R2OpPlugin {
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, CellParserError<'de>> {
        Ok(Self {
            plugin_address: MsgAddress {
                workchain_id: parser.unpack::<i8>()? as i32,
                address: parser.unpack()?,
            },
            amount: parser.unpack_as::<_, Grams>()?,
            query_id: parser.unpack()?,
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
            BoC::from_root(WALLET_V4R2_CODE_CELL.clone()),
            BagOfCellsArgs {
                has_idx: false,
                has_crc32c: true,
            },
        )
        .unwrap();

        let unpacked: BoC = unpack_fully(packed).unwrap();

        let got: Cell = unpacked.single_root().unwrap().parse_fully().unwrap();
        assert_eq!(&got, WALLET_V4R2_CODE_CELL.as_ref());
    }
}
