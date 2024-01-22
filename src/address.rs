use strum::Display;

use crate::{
    CellBuilder, CellParser, ErrorReason, NBits, Result, TLBDeserialize, TLBSerialize,
    TLBSerializeWrapAs,
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

impl TLBSerialize for MsgAddress {
    fn store(&self, builder: &mut CellBuilder) -> Result<()> {
        if self.is_null() {
            builder.store_as::<_, NBits<2>>(MsgAddressTag::Null as u8)?;
        } else {
            builder
                .store_as::<_, NBits<2>>(MsgAddressTag::Std as u8)?
                .store(false)?
                .store(self.workchain_id as i8)?
                .store(self.address)?;
        }
        Ok(())
    }
}

impl<'de> TLBDeserialize<'de> for MsgAddress {
    fn parse(parser: &mut CellParser<'de>) -> Result<Self> {
        let typ = parser.parse()?;
        match typ {
            MsgAddressTag::Null => Ok(Self::NULL),
            MsgAddressTag::Std => {
                // anycast
                let _ = bool::parse(parser)?;
                Ok(Self {
                    workchain_id: parser.parse::<i8>()? as i32,
                    address: parser.parse()?,
                })
            }
            _ => Err(ErrorReason::custom(format!("unsupported address tag: {typ}")).into()),
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

impl TLBSerialize for MsgAddressTag {
    fn store(&self, builder: &mut CellBuilder) -> Result<()> {
        (*self as u8).wrap_as::<NBits<2>>().store(builder)
    }
}

impl<'de> TLBDeserialize<'de> for MsgAddressTag {
    fn parse(parser: &mut CellParser<'de>) -> Result<Self> {
        Ok(match parser.parse_as::<u8, NBits<2>>()? {
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
