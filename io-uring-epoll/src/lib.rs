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
// Filehandle types
//-----------------------------------------------
pub(crate) mod fd;
pub use fd::HandledFd;

//-----------------------------------------------
// Completion types
//-----------------------------------------------
pub mod completion;
#[doc(inline)]
pub use completion::Completion;

//-----------------------------------------------
// Ownership / of long living records / types
//-----------------------------------------------
mod ownership;
#[doc(inline)]
pub use ownership::Owner;

//-----------------------------------------------
// Epoll Uring Handler -> EpollCtl within Uring
//-----------------------------------------------
mod epoll_uring;
#[doc(inline)]
pub use epoll_uring::EpollUringHandler;

//-----------------------------------------------
// Uring Handler -> Core Uring handler
//-----------------------------------------------
mod uring;
#[doc(inline)]
pub use uring::UringHandler;

//-----------------------------------------------
// Epoll Handler -> Epoll Syscalls e.g. wait
//-----------------------------------------------
mod epoll;
#[doc(inline)]
pub use epoll::EpollHandler;

//-----------------------------------------------
// Misc crate-wide private types
//-----------------------------------------------
pub(crate) use std::os::fd::RawFd;
