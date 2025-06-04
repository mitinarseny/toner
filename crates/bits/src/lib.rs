#![doc = include_str!("../README.md")]
//! ## Example
//!
//! Consider the following TL-B schema:
//!
//! ```tlb
//! tag$10 query_id:uint64 amount:(VarUInteger 16) = Hello;
//! ```
//!
//! Let's first define a struct `Hello` that holds these parameters:
//!
//! ```rust
//! # use num_bigint::BigUint;
//! struct Hello {
//!     pub query_id: u64,
//!     pub amount: BigUint,
//! }
//! ```
//!
//! ### **Ser**ialization
//!
//! To be able to **ser**ialize a type to [`BitWriter`](crate::ser::BitWriter),
//! we should implement [`BitPack`](crate::ser::BitPack) on it:
//!
//! ```
//! # use bitvec::{vec::BitVec, order::Msb0};
//! # use num_bigint::BigUint;
//! # use tlbits::{
//! #   r#as::{NBits, VarInt},
//! #   ser::{BitPack, BitWriter, BitWriterExt, pack},
//! #   StringError,
//! # };
//! #
//! # struct Hello {
//! #     pub query_id: u64,
//! #     pub amount: BigUint,
//! # }
//! impl BitPack for Hello {
//!     fn pack<W>(&self, mut writer: W) -> Result<(), W::Error>
//!         where W: BitWriter,
//!     {
//!         writer
//!             // tag$10
//!             .pack_as::<_, NBits<2>>(0b10)?
//!             // query_id:uint64
//!             .pack(self.query_id)?
//!             // amount:(VarUInteger 16)
//!             .pack_as::<_, &VarInt<4>>(&self.amount)?;
//!         Ok(())
//!     }
//! }
//!
//! # fn main() -> Result<(), StringError> {
//! # let mut writer = BitVec::<u8, Msb0>::new().counted();
//! writer.pack(Hello {
//!     query_id: 0,
//!     amount: 1_000u64.into(),
//! })?;
//! # Ok(())
//! # }
//! ```
//!
//! ### **De**serialization
//!
//! To be able to **de**serialize a type from [`BitReader`](crate::de::BitReader),
//! we should implement [`BitUnpack`](crate::de::BitUnpack) on it:
//!
//! ```rust
//! # use bitvec::{vec::BitVec, order::Msb0};
//! # use num_bigint::BigUint;
//! # use tlbits::{
//! #   r#as::{NBits, VarInt},
//! #   de::{BitReaderExt, BitReader, BitUnpack},
//! #   Error,
//! #   ser::{BitPack, BitWriter, BitWriterExt, pack},
//! #   StringError,
//! # };
//! # #[derive(Debug, PartialEq)]
//! # struct Hello {
//! #     pub query_id: u64,
//! #     pub amount: BigUint,
//! # }
//! # impl BitPack for Hello {
//! #     fn pack<W>(&self, mut writer: W) -> Result<(), W::Error>
//! #         where W: BitWriter,
//! #     {
//! #         writer
//! #             // tag$10
//! #             .pack_as::<_, NBits<2>>(0b10)?
//! #             // query_id:uint64
//! #             .pack(self.query_id)?
//! #             // amount:(VarUInteger 16)
//! #             .pack_as::<_, &VarInt<4>>(&self.amount)?;
//! #         Ok(())
//! #     }
//! # }
//! impl<'de> BitUnpack<'de> for Hello {
//!     fn unpack<R>(mut reader: R) -> Result<Self, R::Error>
//!         where R: BitReader<'de>,
//!     {
//!         // tag$10
//!         let tag: u8 = reader.unpack_as::<_, NBits<2>>()?;
//!         if tag != 0b10 {
//!             return Err(Error::custom(format!("unknown tag: {tag:#b}")));
//!         }
//!         Ok(Self {
//!             // query_id:uint64
//!             query_id: reader.unpack()?,
//!             // amount:(VarUInteger 16)
//!             amount: reader.unpack_as::<_, VarInt<4>>()?,
//!         })
//!     }
//! }
//!
//! # fn main() -> Result<(), StringError> {
//! # let orig = Hello {
//! #     query_id: 0,
//! #     amount: 1_000u64.into(),
//! # };
//! # let mut writer = BitVec::<u8, Msb0>::new().counted();
//! # writer.pack(&orig)?;
//! # let mut parser = writer.as_bitslice();
//! let hello: Hello = parser.unpack()?;
//! # assert_eq!(hello, orig);
//! # Ok(())
//! # }
//! ```
pub mod adapters;
pub mod r#as;
pub mod de;
mod error;
pub mod integer;
pub mod ser;

pub use self::error::*;

pub use bitvec;
pub use either;

#[cfg(test)]
mod tests;
