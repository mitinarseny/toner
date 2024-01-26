use strum::Display;
use tlb::{
    BitPack, BitReader, BitReaderExt, BitUnpack, BitWriter, BitWriterExt, Error, NBits, ResultExt,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct MsgAddress {
    pub workchain_id: i32,
    pub address: [u8; 32],
}

impl MsgAddress {
    const NULL: Self = Self {
        workchain_id: 0,
        address: [0; 32],
    };

    #[inline]
    pub fn is_null(&self) -> bool {
        *self == Self::NULL
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
