use anyhow::anyhow;
use nacl::sign::{signature, Keypair};

pub use nacl::sign::{PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyPair {
    /// Secret key of this pair.
    pub secret_key: [u8; SECRET_KEY_LENGTH],

    /// Public key of this pair.
    pub public_key: [u8; PUBLIC_KEY_LENGTH],
}

impl From<Keypair> for KeyPair {
    fn from(Keypair { skey, pkey }: Keypair) -> Self {
        Self {
            secret_key: skey,
            public_key: pkey,
        }
    }
}

impl KeyPair {
    #[inline]
    pub const fn new(
        secret_key: [u8; SECRET_KEY_LENGTH],
        public_key: [u8; PUBLIC_KEY_LENGTH],
    ) -> Self {
        Self {
            secret_key,
            public_key,
        }
    }

    pub fn sign(&self, msg: impl AsRef<[u8]>) -> anyhow::Result<[u8; 64]> {
        signature(msg.as_ref(), self.secret_key.as_slice())
            .map_err(|e| anyhow!("{}", e.message))?
            .try_into()
            .map_err(|sig: Vec<_>| {
                anyhow!(
                    "got signature of a wrong size, expected 64, got: {}",
                    sig.len()
                )
            })
    }
}
