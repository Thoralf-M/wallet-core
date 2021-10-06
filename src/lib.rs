#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]

/// The account module.
pub mod account;
/// The account manager module.
pub mod account_manager;
/// The actor interface for the library.
// #[cfg(feature = "actor")]
// pub mod actor;
/// The client module.
pub mod client;
/// The error module.
pub mod error;
/// The event module.
pub mod events;
/// Signing interfaces.
pub mod signing;
// #[cfg(feature = "storage")]
// /// The storage module.
// pub(crate) mod storage;
// #[cfg(feature = "stronghold")]
// #[cfg_attr(docsrs, doc(cfg(feature = "stronghold")))]
// pub(crate) mod stronghold;

pub use error::Error;
/// The wallet Result type.
pub type Result<T> = std::result::Result<T, Error>;
