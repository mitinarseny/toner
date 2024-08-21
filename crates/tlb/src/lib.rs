#![doc = include_str!("../README.md")]
//! ## Example
//!
//! Consider the following TL-B schema:
//!
//! ```tlb
//! tag$10 query_id:uint64 amount:(VarUInteger 16) payload:(Maybe ^Cell) = Hello;
//! ```
//!
//! Let's first define a struct `Hello` that holds these parameters:
//!
//! ```rust
//! # use num_bigint::BigUint;
//! # use tlb::Cell;
//! struct Hello {
//!     pub query_id: u64,
//!     pub amount: BigUint,
//!     pub payload: Option<Cell>,
//! }
//! ```
//!
//! ### **Ser**ialization
//!
//! To be able to **ser**ialize a type to [`Cell`], we should implement
//! [`CellSerialize`](crate::ser::CellSerialize) on it:
//!
//! ```
//! # use num_bigint::BigUint;
//! # use tlb::{
//! #   r#as::Ref,
//! #   bits::{r#as::{NBits, VarInt}, ser::BitWriterExt},
//! #   Cell,
//! #   ser::{CellSerialize, CellBuilder, CellBuilderError, CellSerializeExt},
//! #   StringError,
//! # };
//! #
//! # struct Hello {
//! #     pub query_id: u64,
//! #     pub amount: BigUint,
//! #     pub payload: Option<Cell>,
//! # }
//! impl CellSerialize for Hello {
//!     fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
//!         builder
//!             // tag$10
//!             .pack_as::<_, NBits<2>>(0b10)?
//!             // query_id:uint64
//!             .pack(self.query_id)?
//!             // amount:(VarUInteger 16)
//!             .pack_as::<_, &VarInt<4>>(&self.amount)?
//!             // payload:(Maybe ^Cell)
//!             .store_as::<_, Option<Ref>>(self.payload.as_ref())?;
//!         Ok(())
//!     }
//! }
//!
//! # fn main() -> Result<(), StringError> {
//! // serialize value into cell
//! let hello = Hello {
//!     query_id: 0,
//!     amount: 1_000u64.into(),
//!     payload: None,
//! };
//! let cell = hello.to_cell()?;
//! # Ok(())
//! # }
//! ```
//!
//! ### **De**serialization
//!
//! To be able to **de**serialize a type from [`Cell`], we should implement
//! [`CellDeserialize`](crate::de::CellDeserialize) on it:
//!
//! ```rust
//! # use num_bigint::BigUint;
//! # use tlb::{
//! #   r#as::{Ref, ParseFully},
//! #   bits::{r#as::{NBits, VarInt}, de::BitReaderExt, ser::BitWriterExt},
//! #   Cell,
//! #   de::{CellDeserialize, CellParser, CellParserError},
//! #   Error,
//! #   ser::{CellSerialize, CellBuilder, CellBuilderError, CellSerializeExt},
//! #   StringError,
//! # };
//! # #[derive(Debug, PartialEq)]
//! # struct Hello {
//! #     pub query_id: u64,
//! #     pub amount: BigUint,
//! #     pub payload: Option<Cell>,
//! # }
//! # impl CellSerialize for Hello {
//! #     fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
//! #         builder
//! #             // tag$10
//! #             .pack_as::<_, NBits<2>>(0b10)?
//! #             // query_id:uint64
//! #             .pack(self.query_id)?
//! #             // amount:(VarUInteger 16)
//! #             .pack_as::<_, &VarInt<4>>(&self.amount)?
//! #             // payload:(Maybe ^Cell)
//! #             .store_as::<_, Option<Ref>>(self.payload.as_ref())?;
//! #         Ok(())
//! #     }
//! # }
//! impl<'de> CellDeserialize<'de> for Hello {
//!     fn parse(parser: &mut CellParser<'de>) -> Result<Self, CellParserError<'de>> {
//!         // tag$10
//!         let tag: u8 = parser.unpack_as::<_, NBits<2>>()?;
//!         if tag != 0b10 {
//!             return Err(Error::custom(format!("unknown tag: {tag:#b}")));
//!         }
//!         Ok(Self {
//!             // query_id:uint64
//!             query_id: parser.unpack()?,
//!             // amount:(VarUInteger 16)
//!             amount: parser.unpack_as::<_, VarInt<4>>()?,
//!             // payload:(Maybe ^Cell)
//!             payload: parser.parse_as::<_, Option<Ref<ParseFully>>>()?,
//!         })
//!     }
//! }
//!
//! # fn main() -> Result<(), StringError> {
//! # let orig = Hello {
//! #     query_id: 0,
//! #     amount: 1_000u64.into(),
//! #     payload: None,
//! # };
//! # let cell = orig.to_cell()?;
//! let mut parser = cell.parser();
//! let hello: Hello = parser.parse()?;
//! # assert_eq!(hello, orig);
//! # Ok(())
//! # }
//! ```
pub mod r#as;
mod cell;
pub mod de;
pub mod ser;

pub use self::cell::*;

pub use tlbits::{self as bits, either, Error, ResultExt, StringError};

#[cfg(test)]
mod tests;
