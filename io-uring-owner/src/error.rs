//! Owner Errors

use core::fmt;
use core::fmt::Display;

/// Errors relating to taking (ownership)
#[derive(Debug)]
pub enum TakeError {
    /// Filling pendign by the User
    PendingFilling,
    /// Already taken, typically internal error.
    AlreadyTaken,
    /// Kernel owns, cannot take.
    KernelOwns,
    /// Buffer is returned but not marked for re-usable
    Returned,
    /// Cannot take multi-buffer-holder, requires one single buffer.
    OnlyOneTakeable,
}

impl Display for TakeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PendingFilling => write!(f, "Pending filling by the user."),
            Self::AlreadyTaken => write!(f, "Kernel owns, cannot take."),
            Self::KernelOwns => write!(
                f,
                "Kernel owns the buffer. Must be claered for re-use first."
            ),
            Self::Returned => write!(f, "Markerd as returnd but not marked for re-use."),
            Self::OnlyOneTakeable => write!(
                f,
                "Cannot take multi-buffer-holder, requires one single buffer."
            ),
        }
    }
}

impl std::error::Error for TakeError {}
