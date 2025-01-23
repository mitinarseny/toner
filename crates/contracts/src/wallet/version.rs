use std::sync::Arc;

use chrono::{DateTime, Utc};
use tlb::{ser::CellSerialize, Cell};
use tlb_ton::{action::SendMsgAction, state_init::StateInit};

use super::PUBLIC_KEY_LENGTH;

/// Version of [`Wallet`](super::Wallet)
pub trait WalletVersion {
    type Data: CellSerialize;
    type SignBody: CellSerialize;
    type ExternalMsgBody: CellSerialize;

    const DEFAULT_WALLET_ID: u32;

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

    /// Wraps signed body into external [`ExternalMsgBody`](WalletVersion::ExternalMsgBody)
    fn wrap_signed_external(body: Self::SignBody, signature: [u8; 64]) -> Self::ExternalMsgBody;

    #[inline]
    fn state_init(
        wallet_id: u32,
        pubkey: [u8; PUBLIC_KEY_LENGTH],
    ) -> StateInit<Arc<Cell>, Self::Data> {
        StateInit {
            code: Some(Self::code()),
            data: Some(Self::init_data(wallet_id, pubkey)),
            ..Default::default()
        }
    }
}
