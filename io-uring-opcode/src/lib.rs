#![warn(
    clippy::unwrap_used,
    missing_docs,
    rust_2018_idioms,
    unused_lifetimes,
    unused_qualifications
)]
#![doc = include_str!("../README.md")]

//***********************************************
// Re-Exports
//***********************************************

//-----------------------------------------------
// All Errors
//-----------------------------------------------
//pub mod error;

//-----------------------------------------------
// UringOpCode Trait
//-----------------------------------------------
mod traits;
pub use traits::*;
