#![warn(
    clippy::unwrap_used,
    missing_docs,
    rust_2018_idioms,
    unused_lifetimes,
    unused_qualifications
)]
#![doc = include_str!("../README.md")]

//! io_uring Owner types

mod error;
use core::{fmt, fmt::Display};
pub use error::TakeError;

/// Ownership denotes where the ownership stands in long living records and where it is important to know it's status.
/// All Accept, EpollCtl and Buffer records have varying dynamics how these
/// are allocated and re-used and this requires a flexible type given re-allocation
/// may be expensive in case of buffers whilst desierable just re-create record in
/// Accept or EpollCtl case.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum Owner {
    /// Record was created (default)
    #[default]
    Created,
    /// Registered (e.g. via ProvideBuffers)
    Registered,
    /// User is filling the created buffer
    Filling,
    /// Taken is owned intermediately internally before used
    Taken,
    /// Record is owned by the Kernel
    Kernel,
    /// Record is returned to the user(space)
    Returned,
    /// Record is marked for re-use (e.g. expensive allocation)
    /// Typical user: BufferRec
    Reusable,
}

impl Owner {
    /// Is the Owner in a state that underlying thing can be taken?
    #[inline]
    pub fn take(&mut self) -> Result<(), TakeError> {
        let r = match self {
            Owner::Created => Ok(()),
            Owner::Registered => Ok(()),
            Owner::Filling => Err(TakeError::PendingFilling),
            Owner::Taken => Err(TakeError::AlreadyTaken),
            Owner::Kernel => Err(TakeError::KernelOwns),
            Owner::Returned => Err(TakeError::Returned),
            Owner::Reusable => Ok(()),
        };
        match r {
            Ok(()) => {
                *self = Owner::Taken;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}

impl Display for Owner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Created => write!(f, "Created"),
            Self::Registered => write!(f, "Registered"),
            Self::Filling => write!(f, "Filling"),
            Self::Taken => write!(f, "Taken"),
            Self::Kernel => write!(f, "Kernel"),
            Self::Returned => write!(f, "Returned"),
            Self::Reusable => write!(f, "Reusable"),
        }
    }
}
