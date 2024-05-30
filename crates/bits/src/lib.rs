pub mod adapters;
pub mod r#as;
pub mod de;
mod either;
mod error;
pub mod integer;
pub mod ser;

pub use self::error::*;

pub use ::either::Either;
pub use bitvec;

#[cfg(test)]
mod tests;
