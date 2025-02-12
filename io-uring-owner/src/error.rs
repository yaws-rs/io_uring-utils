//! Owner Errors

use core::fmt;
use core::fmt::Display;

/// Errors relating to taking (ownership)
#[derive(Debug)]
pub enum TakeError {
    /// Already taken, typically internal error.
    AlreadyTaken,
    /// Kernel owns, cannot take.
    KernelOwns,
    /// User owns and must mark it for re-use first.
    UserOwns,
    /// Cannot take ownership of shared multi-ownership.
    SharedMulti,
    /// Cannot take multi-buffer-holder, requires one single buffer.
    OnlyOneTakeable,
}

impl Display for TakeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyTaken => write!(f, "Kernel owns, cannot take."),
            Self::KernelOwns => write!(
                f,
                "Kernel owns the buffer. Must be claered for re-use first."
            ),
            Self::UserOwns => write!(f, "User owns and must mark it for re-use first."),
            Self::SharedMulti => write!(f, "Cannot take over shared multi-ownership."),
            Self::OnlyOneTakeable => write!(
                f,
                "Cannot take multi-buffer-holder, requires one single buffer."
            ),
        }
    }
}

impl std::error::Error for TakeError {}
