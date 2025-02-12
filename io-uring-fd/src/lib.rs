#![warn(
    clippy::unwrap_used,
    missing_docs,
    rust_2018_idioms,
    unused_lifetimes,
    unused_qualifications
)]
#![doc = include_str!("../README.md")]

use std::os::fd::RawFd;

/// Filehandle registered (or mapped as fixed) with the io_uring API
#[derive(Clone, Debug)]
pub struct RegisteredFd {
    /// Type of registered handle
    pub kind: FdKind,
    /// RawFD of the registered handle
    pub raw_fd: RawFd,
}

impl RegisteredFd {
    /// Create Registered filehandle from RawFd
    #[inline]
    pub fn from_raw(raw_fd: RawFd, kind: FdKind) -> Self {
        Self { kind, raw_fd }
    }
}

/// Type of Fd mainly used by the safe API
#[derive(Clone, Debug)]
pub enum FdKind {
    /// Epoll ctl handle                  
    EpollCtl,
    /// Acceptor handle             
    Acceptor,
    /// Recv Handle
    Recv,
    /// Send handle
    Send,
    /// Recv-Send Handle
    RecvSend,
}
