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
// Handled FD
//-----------------------------------------------
mod handled_fd;
pub use handled_fd::HandledFd;

//-----------------------------------------------
// Epoll Uring Handler -> EpollCtl within Uring
//-----------------------------------------------
pub mod epoll_uring;
#[doc(inline)]
pub use epoll_uring::EpollUringHandler;
