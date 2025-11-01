//! io_uring RingBuf Errors

use core::fmt;
use core::fmt::Display;

use anonymous_mmap::AnonymousMmapError;

/// Errors from the UringRingBuf
#[derive(Debug)]
pub enum RingBufError {
    /// Given / assumed page size must be divisible with the given buffer size
    PageSizeUndivisible,
    /// Error during Mmap from AnonymousMmap
    Mmap(AnonymousMmapError),
    /// io-uring-bearer Related Unregister error
    #[cfg(feature = "bearer")]
    Unregister(crate::RingBufRegistered, std::io::Error),
    /// io-uring-bearer Related Register error
    #[cfg(feature = "bearer")]
    Register(crate::RingBufUnregistered, std::io::Error),
}

impl Display for RingBufError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PageSizeUndivisible => write!(f, "Given / assumed page size must be divisible with the given buffer size"),
            Self::Mmap(am) => write!(f, "mmap error: {}", am),
            #[cfg(feature = "bearer")]
            Self::Register(_, iou) => write!(f, "IoUring Register: {}", iou),
            #[cfg(feature = "bearer")]
            Self::Unregister(_, iou) => write!(f, "IoUring Unregister: {}", iou),            
        }
    }
}

impl core::error::Error for RingBufError {}
