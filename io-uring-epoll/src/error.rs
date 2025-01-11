//! EpollHandler, EpollUringHandler, UringHandler Errors

use core::fmt;
use core::fmt::Display;

use std::error::Error;

use slabbable::SlabbableError;

/// EpollHandler Errors
#[derive(Debug)]
pub enum EpollHandlerError {
    /// epoll_wait Error
    Wait(String),
}

/// Errors from the Epoll Uring Handler
#[derive(Debug)]
pub enum EpollUringHandlerError {
    /// Slab was able to store the value but not get it? This is a bug.
    SlabBugSetGet(&'static str),
    /// EpollCtl OpCode is not supported in your kernel
    NotSupported,
    /// Error probing the support of EpollCtl from kernel
    Probing(String),
    /// Error from the underlying UringHandler
    UringHandler(UringHandlerError),
    /// Error creating epoll handle in Kernel
    EpollCreate1(String),
}

/// Errors from the Uring Handler
#[derive(Debug)]
pub enum UringHandlerError {
    /// Error creating IoUring instance
    IoUringCreate(String),
    /// The Fd is already in the handler and would override existing
    Duplicate,
    /// Something went yoinks in io_uring::IoRing::submit[_and_wait]
    Submission(String),
    /// Something wrong Slab                                        
    Slab(String),
    /// Slab was able to store the value but not get it? This is a bug.
    SlabBugSetGet(&'static str),
    /// Register handles error
    RegisterHandles(String),
    /// Slabbable related error
    Slabbable(SlabbableError),
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
            Self::SlabBugSetGet(s) => write!(f, "Slab Bug: {}", s),
            Self::NotSupported => write!(
                f,
                "EpollCtl io_uring OpCode is not supported in your Kernel"
            ),
            Self::Probing(s) => write!(
                f,
                "Error whilst probing EpollCtl support from kernel: {}",
                s
            ),
            Self::UringHandler(s) => write!(f, "Underlying Uring error: {}", s),
            Self::EpollCreate1(s) => write!(f, "epoll_create1(): {}", s),
        }
    }
}

impl Display for UringHandlerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IoUringCreate(s) => write!(f, "IoUring Create: {}", s),
            Self::Duplicate => write!(f, "The filehandle is already maped in. Possible duplicate?"),
            Self::Submission(s) => write!(f, "Submission: {}", s),
            Self::Slab(s) => write!(f, "Slab: {}", s),
            Self::SlabBugSetGet(s) => write!(f, "Slab Bug: {}", s),
            Self::RegisterHandles(s) => write!(f, "Register Handles: {}", s),
            Self::Slabbable(e) => write!(f, "Slabbable: {}", e),
        }
    }
}

impl Error for EpollUringHandlerError {}
impl Error for UringHandlerError {}
impl Error for EpollHandlerError {}
