//! UringBearer Errors

use io_uring_owner::{Owner, TakeError};

use core::fmt;
use core::fmt::Display;

use io_uring_opcode::OpError;
use slabbable::SlabbableError;

/// Errors from the Uring Handler
#[derive(Debug)]
pub enum UringBearerError {
    /// Error creating IoUring instance
    IoUringCreate(String),
    /// The Fd is already in the handler and would override existing
    Duplicate,
    /// Submission Push error, typically queue full.
    SubmissionPush,
    /// Something went yoinks in io_uring::IoRing::submit[_and_wait]
    Submission(String),
    /// Something wrong Slab                                        
    Slab(String),
    /// Invalid Submission Flags encountered
    SubmissionFlags,
    /// Slab was able to store the value but not get it? This is a bug.
    SlabBugSetGet(&'static str),
    /// Register handles error
    RegisterHandles(String),
    /// Slabbable related error
    Slabbable(SlabbableError),
    /// Supplied value for input key was invalid.
    /// First is the place it happened, Second is the expressed evalaution and Third is the invalid value.
    InvalidParameterI32(&'static str, &'static str, i32),
    /// Cannot directly destroy buffers that are currently owned by the kernel. Use remove_buffers instead.
    BufferNoOwnership(usize),
    /// Buffer does not exist.
    BufferNotExist(usize),
    /// Buffer selected within Buffers does not exist.
    BufferSelectedNotExist(u16),
    /// Cannot take Buffer
    BufferTake(TakeError),
    /// Buffers are not owned by Kernel.
    BufferNotKernelOwned(usize),
    /// Cannot directly destroy futex atomics that are currently owned by the kernel. Use cancel_futex instead.
    FutexNoOwnership(usize),
    /// Futex Atomic does not exist.
    FutexNotExist(usize),
    /// Filehandle must be registered first
    FdNotRegistered(u32),
    /// Kernel has the ownership cannot push to kernel.
    InvalidOwnership(Owner, usize),
    /// Registered filehandles map is full at capacity.
    FdRegisterFull,
    /// Could not register filehandle.
    FdRegisterFail,
    /// OpCode Error
    Op(OpError),
    /// Invalid TargetFd assignment
    InvalidTargetFd(u32),
}

impl Display for UringBearerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IoUringCreate(s) => write!(f, "IoUring Create: {}", s),
            Self::Duplicate => write!(f, "The filehandle is already maped in. Possible duplicate?"),
            Self::SubmissionPush => write!(f, "Submisionn push error. Is the squeue full?"),
            Self::Submission(s) => write!(f, "Submission: {}", s),
            Self::Slab(s) => write!(f, "Slab: {}", s),
            Self::SubmissionFlags => write!(f, "Invalid flags set in Submission"),
            Self::SlabBugSetGet(s) => write!(f, "Slab Bug: {}", s),
            Self::RegisterHandles(s) => write!(f, "Register Handles: {}", s),
            Self::Slabbable(e) => write!(f, "Slabbable: {}", e),
            Self::InvalidParameterI32(at, s, val) => write!(
                f,
                "Invalid input parameter value {} at {} when {}",
                val, at, s
            ),
            Self::InvalidTargetFd(try_slot) => write!(f, "Invalid Target Fd {}", try_slot),
            Self::BufferNoOwnership(idx) => write!(f, "Buffer {} in invalid ownership state", idx),
            Self::BufferNotExist(idx) => write!(f, "Buffer {} does not exist.", idx),
            Self::BufferNotKernelOwned(idx) => {
                write!(f, "Buffer {} is not owned by the kernel.", idx)
            }
            Self::BufferSelectedNotExist(sel_idx) => write!(
                f,
                "Selected {} does not exist within the given buffer.",
                sel_idx
            ),
            Self::BufferTake(take_err) => write!(f, "Unable to take buffer: {}", take_err),
            Self::FutexNoOwnership(idx) => write!(f, "Futex {} in invalid ownership state", idx),
            Self::FutexNotExist(idx) => write!(f, "Futex {} does not exist.", idx),
            Self::FdNotRegistered(idx) => write!(f, "Fixed fd {} is not registered.", idx),
            Self::InvalidOwnership(owner, idx) => {
                write!(f, "Invalid current ownership {} of idx {}", owner, idx)
            }
            Self::FdRegisterFull => write!(
                f,
                "Map holding th registered filehandles is at capacity and cannot add more."
            ),
            Self::FdRegisterFail => write!(f, "Failed to register filehandle."),
            Self::Op(e) => write!(f, "OpCode: {}", e),
        }
    }
}

impl From<OpError> for UringBearerError {
    fn from(e: OpError) -> UringBearerError {
        UringBearerError::Op(e)
    }
}

impl std::error::Error for UringBearerError {}
