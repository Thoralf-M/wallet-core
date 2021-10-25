// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! The IOTA Wallet Library
#![allow(dead_code)]
#![allow(unused_variables)]

/// [`AccountHandle`]: crate::account::handle::AccountHandle
/// The account module. Interaction with an Account happens via an [`AccountHandle`].
pub mod account;
/// The account manager module.
pub mod account_manager;
/// The actor interface for the library. A different way to call the wallet functions, useful for bindings to other
/// languages.
// #[cfg(feature = "actor")]
// pub mod actor;
/// The client module to use iota_client for interactions with the IOTA Tangle.
pub mod client;
/// The error module.
pub mod error;
#[cfg(feature = "events")]
/// The event module.
pub mod events;
/// Signing interfaces for address generation and transaction signing.
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
