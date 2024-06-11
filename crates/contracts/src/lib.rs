#![doc = include_str!("../README.md")]
#[cfg(feature = "wallet")]
#[cfg_attr(docsrs, doc(cfg(feature = "wallet")))]
pub mod wallet;

#[cfg(feature = "jetton")]
#[cfg_attr(docsrs, doc(cfg(feature = "jetton")))]
pub mod jetton;
