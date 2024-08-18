use tlb::{
    bits::{de::BitReaderExt, r#as::NBits, ser::BitWriterExt},
    de::{CellDeserialize, CellParser, CellParserError},
    r#as::Ref,
    ser::{CellBuilder, CellBuilderError, CellSerialize},
    Cell, Error,
};

use crate::{currency::CurrencyCollection, library::LibRef, message::Message};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutAction {
    /// ```tlb
    /// action_send_msg#0ec3c86d mode:(## 8) out_msg:^(MessageRelaxed Any) = OutAction;
    /// ```
    SendMsg(SendMsgAction),

    /// ```tlb
    /// action_set_code#ad4de08e new_code:^Cell = OutAction;
    /// ```
    SetCode(Cell),

    /// ```tlb
    /// action_reserve_currency#36e6b809 mode:(## 8) currency:CurrencyCollection = OutAction;
    /// ```
    ReserveCurrency(ReserveCurrencyAction),

    /// ```tlb
    /// action_change_library#26fa1dd4 mode:(## 7) libref:LibRef = OutAction;
    /// ```
    ChangeLibrary(ChangeLibraryAction),
}

impl OutAction {
    const SEND_MSG_PREFIX: u32 = 0x0ec3c86d;
    const SET_CODE_PREFIX: u32 = 0xad4de08e;
    const RESERVE_CURRENCY_PREFIX: u32 = 0x36e6b809;
    const CHANGE_LIBRARY_PREFIX: u32 = 0x26fa1dd4;
}

impl CellSerialize for OutAction {
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        match self {
            OutAction::SendMsg(action) => builder.pack(Self::SEND_MSG_PREFIX)?.store(action)?,
            OutAction::SetCode(new_code) => builder
                .pack(Self::SET_CODE_PREFIX)?
                .store_as::<_, Ref>(new_code)?,
            OutAction::ReserveCurrency(action) => {
                builder.pack(Self::RESERVE_CURRENCY_PREFIX)?.store(action)?
            }
            OutAction::ChangeLibrary(action) => {
                builder.pack(Self::CHANGE_LIBRARY_PREFIX)?.store(action)?
            }
        };
        Ok(())
    }
}

impl<'de> CellDeserialize<'de> for OutAction {
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, CellParserError<'de>> {
        Ok(match parser.unpack()? {
            Self::SEND_MSG_PREFIX => Self::SendMsg(parser.parse()?),
            Self::SET_CODE_PREFIX => Self::SetCode(parser.parse_as::<_, Ref>()?),
            Self::RESERVE_CURRENCY_PREFIX => Self::ReserveCurrency(parser.parse()?),
            Self::CHANGE_LIBRARY_PREFIX => Self::ChangeLibrary(parser.parse()?),
            prefix => return Err(Error::custom(format!("unknown prefix {prefix:#0x}"))),
        })
    }
}

/// ```tlb
/// action_send_msg#0ec3c86d mode:(## 8) out_msg:^(MessageRelaxed Any) = OutAction;
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendMsgAction<T = Cell, IC = Cell, ID = Cell> {
    /// See <https://docs.ton.org/develop/func/stdlib#send_raw_message>
    pub mode: u8,
    pub message: Message<T, IC, ID>,
}

impl<T, IC, ID> CellSerialize for SendMsgAction<T, IC, ID>
where
    T: CellSerialize,
    IC: CellSerialize,
    ID: CellSerialize,
{
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        builder.pack(self.mode)?.store_as::<_, Ref>(&self.message)?;
        Ok(())
    }
}

impl<'de, T, IC, ID> CellDeserialize<'de> for SendMsgAction<T, IC, ID>
where
    T: CellDeserialize<'de>,
    IC: CellDeserialize<'de>,
    ID: CellDeserialize<'de>,
{
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, CellParserError<'de>> {
        Ok(Self {
            mode: parser.unpack()?,
            message: parser.parse()?,
        })
    }
}

/// ```tlb
/// action_reserve_currency#36e6b809 mode:(## 8) currency:CurrencyCollection = OutAction;
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReserveCurrencyAction {
    pub mode: u8,
    pub currency: CurrencyCollection,
}

impl CellSerialize for ReserveCurrencyAction {
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        builder.pack(self.mode)?.store(&self.currency)?;
        Ok(())
    }
}

impl<'de> CellDeserialize<'de> for ReserveCurrencyAction {
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, CellParserError<'de>> {
        Ok(Self {
            mode: parser.unpack()?,
            currency: parser.parse()?,
        })
    }
}

/// ```tlb
/// action_change_library#26fa1dd4 mode:(## 7) libref:LibRef = OutAction;
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeLibraryAction<R = Cell> {
    pub mode: u8,
    pub libref: LibRef<R>,
}

impl<R> CellSerialize for ChangeLibraryAction<R>
where
    R: CellSerialize,
{
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        builder
            .pack_as::<_, NBits<7>>(self.mode)?
            .store(&self.libref)?;
        Ok(())
    }
}

impl<'de, R> CellDeserialize<'de> for ChangeLibraryAction<R>
where
    R: CellDeserialize<'de>,
{
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, CellParserError<'de>> {
        Ok(Self {
            mode: parser.unpack_as::<_, NBits<7>>()?,
            libref: parser.parse()?,
        })
    }
}
