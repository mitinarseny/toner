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
