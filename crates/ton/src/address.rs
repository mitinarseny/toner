use core::{
    fmt::{self, Debug, Display},
    str::FromStr,
};

use base64::{
    engine::general_purpose::STANDARD_NO_PAD, engine::general_purpose::URL_SAFE_NO_PAD, Engine,
};
use crc::Crc;
use strum::Display;
use tlb::{
    BitPack, BitReader, BitReaderExt, BitUnpack, BitWriter, BitWriterExt, Error, NBits, ResultExt,
};

const CRC_16_XMODEM: Crc<u16> = Crc::<u16>::new(&crc::CRC_16_XMODEM);

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct MsgAddress {
    pub workchain_id: i32,
    pub address: [u8; 32],
}

impl MsgAddress {
    const NULL: Self = Self {
        workchain_id: 0,
        address: [0; 32],
    };

    pub fn from_hex(s: impl AsRef<str>) -> Result<Self, String> {
        let s = s.as_ref();
        let (workchain, addr) = s.split_once(':').ok_or("wrong format".to_string())?;
        let workchain_id = workchain.parse::<i32>().map_err(|err| err.to_string())?;
        let mut address = [0; 32];
        hex::decode_to_slice(addr, &mut address).map_err(|err| err.to_string())?;
        Ok(Self {
            workchain_id,
            address,
        })
    }

    pub fn to_hex(&self) -> String {
        format!("{}:{}", self.workchain_id, hex::encode(self.address))
    }

    pub fn from_base64_url(s: impl AsRef<str>) -> Result<Self, String> {
        Self::from_base64_url_flags(s).map(|(addr, _, _)| addr)
    }

    pub fn from_base64_url_flags(s: impl AsRef<str>) -> Result<(Self, bool, bool), String> {
        Self::from_base64_repr(URL_SAFE_NO_PAD, s)
    }

    pub fn from_base64_std(s: impl AsRef<str>) -> Result<Self, String> {
        Self::from_base64_std_flags(s).map(|(addr, _, _)| addr)
    }

    pub fn from_base64_std_flags(s: impl AsRef<str>) -> Result<(Self, bool, bool), String> {
        Self::from_base64_repr(STANDARD_NO_PAD, s)
    }

    pub fn to_base64_url(self) -> String {
        self.to_base64_url_flags(false, false)
    }

    pub fn to_base64_url_flags(self, non_bounceable: bool, non_production: bool) -> String {
        self.to_base64_flags(non_bounceable, non_production, URL_SAFE_NO_PAD)
    }

    pub fn to_base64_std(self) -> String {
        self.to_base64_std_flags(false, false)
    }

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
    ) -> Result<(Self, bool, bool), String> {
        let s = s.as_ref();
        if s.len() != 48 {
            return Err("invalid length".to_string());
        }
        let mut bytes = [0; 36];
        engine
            .decode_slice(s, &mut bytes)
            .map_err(|err| err.to_string())?;

        let (non_production, non_bounceable) = match bytes[0] {
            0x11 => (false, false),
            0x51 => (false, true),
            0x91 => (true, false),
            0xD1 => (true, true),
            _ => return Err("Invalid base64src address: Wrong tag byte".to_string()),
        };
        let workchain_id = bytes[1] as i8 as i32;
        let calc_crc = CRC_16_XMODEM.checksum(&bytes[0..34]);
        let addr_crc = ((bytes[34] as u16) << 8) | bytes[35] as u16;
        if calc_crc != addr_crc {
            return Err("Invalid base64src address: CRC mismatch".to_string());
        }
        let mut address = [0; 32];
        address.copy_from_slice(&bytes[2..34]);
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

    #[inline]
    pub fn is_null(&self) -> bool {
        *self == Self::NULL
    }
}

impl Debug for MsgAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for MsgAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.to_base64_url().as_str())
    }
}

impl FromStr for MsgAddress {
    type Err = String;

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
            writer
                .pack_as::<_, NBits<2>>(MsgAddressTag::Null as u8)
                .context("tag")?;
        } else {
            writer
                .pack_as::<_, NBits<2>>(MsgAddressTag::Std as u8)
                .context("tag")?
                .pack(false)
                .context("anycast")?
                .pack(self.workchain_id as i8)
                .context("workchain_id")?
                .pack(self.address)
                .context("address")?;
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
        match reader.unpack().context("tag")? {
            MsgAddressTag::Null => Ok(Self::NULL),
            MsgAddressTag::Std => {
                reader.skip(1).context("anycast")?;
                Ok(Self {
                    workchain_id: reader.unpack::<i8>().context("workchain_id")? as i32,
                    address: reader.unpack().context("address")?,
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

#[cfg(feature = "tonlib")]
mod tonlib {
    use tonlib::address::TonAddress;

    use crate::MsgAddress;

    impl From<TonAddress> for MsgAddress {
        fn from(address: TonAddress) -> Self {
            Self {
                workchain_id: address.workchain,
                address: address.hash_part,
            }
        }
    }

    impl From<MsgAddress> for TonAddress {
        fn from(address: MsgAddress) -> Self {
            Self {
                workchain: address.workchain_id,
                hash_part: address.address,
            }
        }
    }
}
