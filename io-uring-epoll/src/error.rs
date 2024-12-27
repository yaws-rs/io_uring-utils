//! EpollHandler, EpollUringHandler, UringHandler Errors

/// EpollHandler Errors
#[derive(Debug)]
pub enum EpollHandlerError {
    /// epoll_wait Error
    Wait(String),
}

/// Errors from the handler           
#[derive(Debug)]
pub enum EpollUringHandlerError {
    /// Error creating IoUring instance
    IoUringCreate(String),
    /// Error creating epoll handle in Kernel
    EpollCreate1(String),
    /// EpollCtl OpCode is not supported in your kernel
    NotSupported,
    /// Error probing the support of EpollCtl from kernel
    Probing(String),
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
}

use core::fmt::{Display, Formatter};

impl Display for EpollHandlerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Wait(s) => write!(f, "EpollHandlerWait: {}", s),
        }
    }
}

impl Display for EpollUringHandlerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::IoUringCreate(s) => write!(f, "IoUring Create: {}", s),
            Self::EpollCreate1(s) => write!(f, "epoll_create1(): {}", s),
            Self::NotSupported => write!(
                f,
                "EpollCtl io_uring OpCode is not supported in your Kernel"
            ),
            Self::Probing(s) => write!(
                f,
                "Error whilst probing EpollCtl support from kernel: {}",
                s
            ),
            Self::Duplicate => write!(f, "The filehandle is already maped in. Possible duplicate?"),
            Self::Submission(s) => write!(f, "Submission: {}", s),
            Self::Slab(s) => write!(f, "Slab: {}", s),
            Self::SlabBugSetGet(s) => write!(f, "Slab Bug: {}", s),
            Self::RegisterHandles(s) => write!(f, "Register Handles: {}", s),
        }
    }
}
