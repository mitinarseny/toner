use std::{collections::HashSet, str::FromStr};

use anyhow::anyhow;
use hmac::{Hmac, Mac};
use lazy_static::lazy_static;
use nacl::sign::generate_keypair;
use pbkdf2::{password_hash::Output, pbkdf2_hmac};
use sha2::Sha512;

pub use nacl::sign::Keypair;

lazy_static! {
    static ref WORDLIST_EN: HashSet<&'static str> = include_str!("./wordlist_en.txt")
        .split_whitespace()
        .filter(|w| !w.is_empty())
        .collect();
}

#[derive(Debug, Clone)]
pub struct Mnemonic([&'static str; 24]);

impl Mnemonic {
    const PBKDF_ITERATIONS: u32 = 100000;

    pub fn generate_keypair(&self, password: impl Into<Option<String>>) -> anyhow::Result<Keypair> {
        let entropy = self.entropy(password)?;
        let seed = Self::pbkdf2_sha512(
            entropy.as_slice(),
            "TON default seed",
            Self::PBKDF_ITERATIONS,
            64,
        )?;
        Ok(generate_keypair(&seed[0..32]))
    }

    fn entropy(&self, password: impl Into<Option<String>>) -> anyhow::Result<[u8; 64]> {
        let mut mac = Hmac::<Sha512>::new_from_slice(self.0.join(" ").as_bytes())?;
        if let Some(password) = password.into() {
            mac.update(password.as_bytes());
        }
        Ok(mac.finalize().into_bytes().into())
    }

    fn pbkdf2_sha512(
        key: &[u8],
        salt: &str,
        rounds: u32,
        output_length: usize,
    ) -> anyhow::Result<Vec<u8>> {
        let output = Output::init_with(output_length, |out| {
            pbkdf2_hmac::<Sha512>(key, salt.as_bytes(), rounds, out);
            Ok(())
        })
        .map_err(|err| anyhow!("{err}"))?;
        Ok(output.as_bytes().to_vec())
    }
}

impl FromStr for Mnemonic {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut words = Vec::new();
        for w in s.split_whitespace() {
            let Some(word) = WORDLIST_EN.get(w.to_lowercase().as_str()) else {
                return Err(anyhow!("word '{w}' is not in the allowed list"));
            };
            words.push(*word);
        }
        Ok(Self(words.try_into().map_err(|words: Vec<_>| {
            anyhow!(
                "mnemonic must consist from exactly 24 words, got: {}",
                words.len()
            )
        })?))
    }
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use super::*;

    #[test]
    fn key_pair() {
        let mnemonic: Mnemonic =
                "dose ice enrich trigger test dove century still betray gas diet dune use other base gym mad law immense village world example praise game"
            .parse().unwrap();
        let kp = mnemonic.generate_keypair(None).unwrap();
        assert_eq!(kp.skey, hex!("119dcf2840a3d56521d260b2f125eedc0d4f3795b9e627269a4b5a6dca8257bdc04ad1885c127fe863abb00752fa844e6439bb04f264d70de7cea580b32637ab"));
    }
}
