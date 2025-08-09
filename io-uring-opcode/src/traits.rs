//! UringOpCode represents the API between the io-uring-bearer and the
//! individual io_uring OpCode handlers which are sepated into different
//! crates.

#[cfg(feature = "accept_multi")]
mod accept_multi;
#[cfg(feature = "accept_multi")]
pub use accept_multi::OpExtAcceptMulti;

#[cfg(feature = "epoll")]
mod epoll_ctl;
#[cfg(feature = "epoll")]
pub use epoll_ctl::OpExtEpollCtl;

#[cfg(feature = "connect")]
mod connect;
#[cfg(feature = "connect")]
pub use connect::OpExtConnect;

#[cfg(feature = "socket")]
mod socket;
#[cfg(feature = "socket")]
pub use socket::OpExtSocket;

use core::pin::Pin;
use io_uring_owner::Owner;

use crate::OpError;

/// Pending Completion type implemented by the OpCode handlers.
pub trait OpCompletion {
    /// It is recommended that you use a harmonized error type but is not mandatory.
    type Error;
    /// Provide the squeue entry
    fn entry(&self) -> io_uring::squeue::Entry;
    /// Get the current Owner
    fn owner(&self) -> Owner;
    /// Force set the owner to Kernel
    fn force_owner_kernel(&mut self) -> bool;
}

/// The contracting type between io-uring-bearer and all the opcodes it can carry.
/// Implement this type in the individual opcodes that can be used in the bearer.
pub trait OpCode<C: OpCompletion> {
    /// It is recommended that you use a harmonized error type but is not mandatory.
    // Turn the abstract OpCoe into Submission that will be pending completion.
    // io-uring-bearer will call this in order to convert the higher level type into actual submission.
    fn submission(self) -> Result<C, OpError>;
    /// io-uring-bearer will call this upno completion
    fn completion(&mut self, _: Pin<&mut C>) -> Result<(), OpError>;
}
