use core::{
    fmt::{self, Debug, Display},
    str::FromStr,
};

use base64::{
    Engine, engine::general_purpose::STANDARD_NO_PAD, engine::general_purpose::URL_SAFE_NO_PAD,
};
use crc::Crc;
use digest::{Digest, Output};
use strum::Display;
use tlb::{
    Context, Error, StringError,
    bits::{
        r#as::{NBits, VarBits},
        bitvec::{order::Msb0, vec::BitVec},
        de::{BitReader, BitReaderExt, BitUnpack},
        ser::{BitPack, BitWriter, BitWriterExt},
    },
    ser::{CellBuilderError, CellSerialize, CellSerializeExt},
};

use crate::state_init::StateInit;

const CRC_16_XMODEM: Crc<u16> = Crc::<u16>::new(&crc::CRC_16_XMODEM);

/// [MsgAddress](https://docs.ton.org/develop/data-formats/msg-tlb#msgaddressext-tl-b)
/// ```tlb
/// addr_none$00 = MsgAddressExt;
/// addr_extern$01 len:(## 9) external_address:(bits len) = MsgAddressExt;
///
/// addr_std$10 anycast:(Maybe Anycast)
/// workchain_id:int8 address:bits256  = MsgAddressInt;
/// addr_var$11 anycast:(Maybe Anycast) addr_len:(## 9)
/// workchain_id:int32 address:(bits addr_len) = MsgAddressInt;
///
/// _ _:MsgAddressInt = MsgAddress;
/// _ _:MsgAddressExt = MsgAddress;
/// ```
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[cfg_attr(
    feature = "serde",
    derive(::serde_with::SerializeDisplay, ::serde_with::DeserializeFromStr)
)]
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MsgAddress {
    #[cfg_attr(
        feature = "arbitrary",
        arbitrary(with = |u: &mut ::arbitrary::Unstructured| u.int_in_range(i8::MIN as i32..=i8::MAX as i32))
    )]
    pub workchain_id: i32,
    pub address: [u8; 32],
}

impl MsgAddress {
    pub const NULL: Self = Self {
        workchain_id: 0,
        address: [0; 32],
    };

    /// [Derive](https://docs.ton.org/learn/overviews/addresses#address-of-smart-contract)
    /// [`MsgAddress`] of a smart-contract by its workchain and [`StateInit`]
    #[cfg(feature = "sha2")]
    #[inline]
    pub fn derive<C, D>(
        workchain_id: i32,
        state_init: StateInit<C, D>,
    ) -> Result<Self, CellBuilderError>
    where
        C: CellSerialize,
        D: CellSerialize,
    {
        Self::derive_digest::<C, D, sha2::Sha256>(workchain_id, state_init)
    }

    #[inline]
    pub fn derive_digest<C, D, H>(
        workchain_id: i32,
        state_init: StateInit<C, D>,
    ) -> Result<Self, CellBuilderError>
    where
        C: CellSerialize,
        D: CellSerialize,
        H: Digest,
        Output<H>: Into<[u8; 32]>,
    {
        Ok(Self {
            workchain_id,
            address: state_init.to_cell()?.hash_digest::<H>(),
        })
    }

    pub fn from_hex(s: impl AsRef<str>) -> Result<Self, StringError> {
        let s = s.as_ref();
        let (workchain, addr) = s
            .split_once(':')
            .ok_or_else(|| Error::custom("wrong format"))?;
        let workchain_id = workchain.parse::<i32>().map_err(Error::custom)?;
        let mut address = [0; 32];
        hex::decode_to_slice(addr, &mut address).map_err(Error::custom)?;
        Ok(Self {
            workchain_id,
            address,
        })
    }

    /// [Raw Address](https://docs.ton.org/learn/overviews/addresses#raw-address)
    /// representation
    #[inline]
    pub fn to_hex(&self) -> String {
        format!("{}:{}", self.workchain_id, hex::encode(self.address))
    }

    /// Shortcut for [`.from_base64_url_flags()?.0`](MsgAddress::from_base64_url_flags)
    #[inline]
    pub fn from_base64_url(s: impl AsRef<str>) -> Result<Self, StringError> {
        Self::from_base64_url_flags(s).map(|(addr, _, _)| addr)
    }

    /// Parse address from URL-base64
    /// [user-friendly](https://docs.ton.org/learn/overviews/addresses#user-friendly-address)
    /// representation and its flags: `(address, non_bouncible, non_production)`
    #[inline]
    pub fn from_base64_url_flags(s: impl AsRef<str>) -> Result<(Self, bool, bool), StringError> {
        Self::from_base64_repr(URL_SAFE_NO_PAD, s)
    }

    /// Shortcut for [`.from_base64_std_flags()?.0`](MsgAddress::from_base64_std_flags)
    #[inline]
    pub fn from_base64_std(s: impl AsRef<str>) -> Result<Self, StringError> {
        Self::from_base64_std_flags(s).map(|(addr, _, _)| addr)
    }

    /// Parse address from standard base64
    /// [user-friendly](https://docs.ton.org/learn/overviews/addresses#user-friendly-address)
    /// representation and its flags: `(address, non_bouncible, non_production)`
    #[inline]
    pub fn from_base64_std_flags(s: impl AsRef<str>) -> Result<(Self, bool, bool), StringError> {
        Self::from_base64_repr(STANDARD_NO_PAD, s)
    }

    /// Shortcut for [`.to_base64_url_flags(false, false)`](MsgAddress::to_base64_url_flags)
    #[inline]
    pub fn to_base64_url(self) -> String {
        self.to_base64_url_flags(false, false)
    }

    /// Encode address as URL base64
    #[inline]
    pub fn to_base64_url_flags(self, non_bounceable: bool, non_production: bool) -> String {
        self.to_base64_flags(non_bounceable, non_production, URL_SAFE_NO_PAD)
    }

    /// Shortcut for [`.to_base64_std_flags(false, false)`](MsgAddress::to_base64_std_flags)
    #[inline]
    pub fn to_base64_std(self) -> String {
        self.to_base64_std_flags(false, false)
    }

    /// Encode address as standard base64
    #[inline]
    pub fn to_base64_std_flags(self, non_bounceable: bool, non_production: bool) -> String {
        self.to_base64_flags(non_bounceable, non_production, STANDARD_NO_PAD)
    }

    /// Parses standard base64 representation of an address
    ///
    /// # Returns
    /// the address, non-bounceable flag, non-production flag.
    fn from_base64_repr(
        engine: impl Engine,
        s: impl AsRef<str>,
    ) -> Result<(Self, bool, bool), StringError> {
        let mut bytes = [0; 36];
        if engine
            .decode_slice(s.as_ref(), &mut bytes)
            .map_err(Error::custom)
            .context("base64")?
            != bytes.len()
        {
            return Err(Error::custom("invalid length"));
        };

        let (non_production, non_bounceable) = match bytes[0] {
            0x11 => (false, false),
            0x51 => (false, true),
            0x91 => (true, false),
            0xD1 => (true, true),
            flags => return Err(Error::custom(format!("unsupported flags: {flags:#x}"))),
        };
        let workchain_id = bytes[1] as i8 as i32;
        let crc = ((bytes[34] as u16) << 8) | bytes[35] as u16;
        if crc != CRC_16_XMODEM.checksum(&bytes[0..34]) {
            return Err(Error::custom("CRC mismatch"));
        }
        let mut address = [0_u8; 32];
        address.clone_from_slice(&bytes[2..34]);
        Ok((
            Self {
                workchain_id,
                address,
            },
            non_bounceable,
            non_production,
        ))
    }

    fn to_base64_flags(
        self,
        non_bounceable: bool,
        non_production: bool,
        engine: impl Engine,
    ) -> String {
        let mut bytes = [0; 36];
        let tag: u8 = match (non_production, non_bounceable) {
            (false, false) => 0x11,
            (false, true) => 0x51,
            (true, false) => 0x91,
            (true, true) => 0xD1,
        };
        bytes[0] = tag;
        bytes[1] = (self.workchain_id & 0xff) as u8;
        bytes[2..34].clone_from_slice(&self.address);
        let crc = CRC_16_XMODEM.checksum(&bytes[0..34]);
        bytes[34] = ((crc >> 8) & 0xff) as u8;
        bytes[35] = (crc & 0xff) as u8;
        engine.encode(bytes)
    }

    /// Returns whether this address is [`NULL`](MsgAddress::NULL)
    #[inline]
    pub fn is_null(&self) -> bool {
        *self == Self::NULL
    }
}

impl Debug for MsgAddress {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.to_hex().as_str())
    }
}

impl Display for MsgAddress {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.to_base64_url().as_str())
    }
}

impl FromStr for MsgAddress {
    type Err = StringError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() == 48 {
            if s.contains(['-', '_']) {
                Self::from_base64_url(s)
            } else {
                Self::from_base64_std(s)
            }
        } else {
            Self::from_hex(s)
        }
    }
}

impl BitPack for MsgAddress {
    #[inline]
    fn pack<W>(&self, mut writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        if self.is_null() {
            writer.pack(MsgAddressTag::Null)?;
        } else {
            writer
                .pack(MsgAddressTag::Std)?
                // anycast:(Maybe Anycast)
                .pack::<Option<Anycast>>(None)?
                // workchain_id:int8
                .pack(self.workchain_id as i8)?
                // address:bits256
                .pack(self.address)?;
        }
        Ok(())
    }
}

impl BitUnpack for MsgAddress {
    #[inline]
    fn unpack<R>(mut reader: R) -> Result<Self, R::Error>
    where
        R: BitReader,
    {
        match reader.unpack()? {
            MsgAddressTag::Null => Ok(Self::NULL),
            MsgAddressTag::Std => {
                // anycast:(Maybe Anycast)
                let _: Option<Anycast> = reader.unpack()?;
                Ok(Self {
                    // workchain_id:int8
                    workchain_id: reader.unpack::<i8>()? as i32,
                    // address:bits256
                    address: reader.unpack()?,
                })
            }
            MsgAddressTag::Var => {
                // anycast:(Maybe Anycast)
                let _: Option<Anycast> = reader.unpack()?;
                // addr_len:(## 9)
                let addr_len: u16 = reader.unpack_as::<_, NBits<9>>()?;
                if addr_len != 256 {
                    // TODO
                    return Err(Error::custom(format!(
                        "only 256-bit addresses are supported for addr_var$11, got {addr_len} bits"
                    )));
                }
                Ok(Self {
                    // workchain_id:int32
                    workchain_id: reader.unpack()?,
                    // address:(bits addr_len)
                    address: reader.unpack()?,
                })
            }
            tag => Err(Error::custom(format!("unsupported address tag: {tag}"))),
        }
    }
}

#[derive(Clone, Copy, Display)]
#[repr(u8)]
enum MsgAddressTag {
    #[strum(serialize = "addr_none$00")]
    Null,
    #[strum(serialize = "addr_extern$01")]
    Extern,
    #[strum(serialize = "addr_std$10")]
    Std,
    #[strum(serialize = "addr_var$11")]
    Var,
}

impl BitPack for MsgAddressTag {
    #[inline]
    fn pack<W>(&self, mut writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        writer.pack_as::<_, NBits<2>>(*self as u8)?;
        Ok(())
    }
}

impl BitUnpack for MsgAddressTag {
    #[inline]
    fn unpack<R>(mut reader: R) -> Result<Self, R::Error>
    where
        R: BitReader,
    {
        Ok(match reader.unpack_as::<u8, NBits<2>>()? {
            0b00 => Self::Null,
            0b01 => Self::Extern,
            0b10 => Self::Std,
            0b11 => Self::Var,
            _ => unreachable!(),
        })
    }
}

/// ```tlb
/// anycast_info$_ depth:(#<= 30) { depth >= 1 } rewrite_pfx:(bits depth) = Anycast;
/// ```
pub struct Anycast {
    pub rewrite_pfx: BitVec<u8, Msb0>,
}

impl BitPack for Anycast {
    fn pack<W>(&self, mut writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        if self.rewrite_pfx.is_empty() {
            return Err(Error::custom("depth >= 1"));
        }
        writer.pack_as::<_, VarBits<5>>(&self.rewrite_pfx)?;
        Ok(())
    }
}

impl BitUnpack for Anycast {
    fn unpack<R>(mut reader: R) -> Result<Self, R::Error>
    where
        R: BitReader,
    {
        let rewrite_pfx = reader.unpack_as::<_, VarBits<5>>()?;
        if rewrite_pfx.is_empty() {
            return Err(Error::custom("depth >= 1"));
        }
        Ok(Self { rewrite_pfx })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_address() {
        let _: MsgAddress = "EQBGXZ9ddZeWypx8EkJieHJX75ct0bpkmu0Y4YoYr3NM0Z9e"
            .parse()
            .unwrap();
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde() {
        use serde_json::json;

        let _: MsgAddress =
            serde_json::from_value(json!("EQBGXZ9ddZeWypx8EkJieHJX75ct0bpkmu0Y4YoYr3NM0Z9e"))
                .unwrap();
    }
}
