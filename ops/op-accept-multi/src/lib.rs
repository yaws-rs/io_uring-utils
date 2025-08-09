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
// Record Types
//-----------------------------------------------
mod accept_multi;
pub use accept_multi::AcceptMulti;

//-----------------------------------------------
// Misc crate-wide private types
//-----------------------------------------------
