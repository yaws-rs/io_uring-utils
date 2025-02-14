//! EpollHandler, EpollUringHandler, UringHandler Errors

use core::fmt;
use core::fmt::Display;

use io_uring_bearer::error::UringBearerError;

/// EpollCtl Errors
#[derive(Debug)]
pub enum EpollCtlError {}

impl Display for EpollCtlError {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

/// EpollHandler Errors
#[derive(Debug)]
pub enum EpollHandlerError {
    /// epoll_wait Error
    Wait(String),
}

/// Errors from the Epoll Uring Handler
#[derive(Debug)]
pub enum EpollUringHandlerError {
    /// EpollCtl OpCode is not supported in your kernel
    NotSupported,
    /// Error probing the support of EpollCtl from kernel
    Probing(String),
    /// Error from the underlying UringHandler
    UringBearer(UringBearerError),
    /// Error creating epoll handle in Kernel
    EpollCreate1(String),
}

impl Display for EpollHandlerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Wait(s) => write!(f, "EpollHandlerWait: {}", s),
        }
    }
}

impl Display for EpollUringHandlerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotSupported => write!(
                f,
                "EpollCtl io_uring OpCode is not supported in your Kernel"
            ),
            Self::Probing(s) => write!(
                f,
                "Error whilst probing EpollCtl support from kernel: {}",
                s
            ),
            Self::UringBearer(s) => write!(f, "Underlying Uring error: {}", s),
            Self::EpollCreate1(s) => write!(f, "epoll_create1(): {}", s),
        }
    }
}

impl std::error::Error for EpollCtlError {}
impl std::error::Error for EpollUringHandlerError {}
impl std::error::Error for EpollHandlerError {}
