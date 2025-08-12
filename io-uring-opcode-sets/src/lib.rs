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
//mod error;
//pub use error::*;

//-----------------------------------------------
// Wrapper
//-----------------------------------------------
mod wrapper;
#[doc(inline)]
pub use wrapper::Wrapper;
