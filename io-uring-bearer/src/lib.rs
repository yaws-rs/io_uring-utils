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
pub use io_uring;

//-----------------------------------------------
// All Errors
//-----------------------------------------------
pub mod error;

//-----------------------------------------------
// Slab (Slotmap) types
//-----------------------------------------------
pub mod slab;

//-----------------------------------------------
// Fixed / registered filehandles etc.
//-----------------------------------------------
pub(crate) mod fixed;

//-----------------------------------------------
// Capacity types
//-----------------------------------------------
mod capacity;
pub use capacity::BearerCapacityKind;

//-----------------------------------------------
// Submission types
//-----------------------------------------------
mod submission;
#[doc(inline)]
pub use submission::SubmissionFlags;
#[doc(inline)]
pub use submission::TargetFd;

//-----------------------------------------------
// Completion types
//-----------------------------------------------
pub mod completion;
#[doc(inline)]
pub use completion::Completion;

//-----------------------------------------------
// Uring Handler -> Core Uring handler
//-----------------------------------------------
mod uring;
#[doc(inline)]
pub use uring::UringBearer;

//-----------------------------------------------
// Misc crate-wide private types
//-----------------------------------------------
pub(crate) use std::os::fd::RawFd;
