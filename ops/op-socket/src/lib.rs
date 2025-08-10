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
// Socket Record Types
//-----------------------------------------------
mod socket;
#[doc(inline)]
pub use socket::Socket;

//-----------------------------------------------
// Re-export associated types
//-----------------------------------------------
#[doc(inline)]
pub use io_uring_opcode::types::TargetFdType;

//-----------------------------------------------
// Misc crate-wide private types
//-----------------------------------------------
