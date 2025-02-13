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
// Filehandle types
//-----------------------------------------------
pub(crate) mod handled_fd;
pub use handled_fd::HandledFd;

//-----------------------------------------------
// Epoll Uring Handler -> EpollCtl within Uring
//-----------------------------------------------
mod epoll_uring;
#[doc(inline)]
pub use epoll_uring::EpollUringHandler;

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
