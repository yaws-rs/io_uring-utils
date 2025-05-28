#![warn(
    clippy::unwrap_used,
    missing_docs,
    rust_2018_idioms,
    unused_lifetimes,
    unused_qualifications
)]
#![doc = include_str!("../README.md")]

//-----------------------------------------------
// All Errors
//-----------------------------------------------
mod error;
#[doc(inline)]
pub use error::*;

//-----------------------------------------------
// Connect Record Types
//-----------------------------------------------
mod connect;
pub use connect::Connect;

//-----------------------------------------------
// Misc crate-wide private types
//-----------------------------------------------
pub(crate) use std::os::fd::RawFd;
