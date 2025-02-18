//! EpollCtl OpCode Handler

use crate::error::EpollUringHandlerError;
use crate::RawFd;

use io_uring_bearer::UringBearer;
use io_uring_fd::{FdKind, RegisteredFd};

use io_uring_opcode::OpCompletion;

/// EpollCtlHandler
pub struct EpollUringHandler {
    /// EpollCtl FD
    pub(crate) epfd: RawFd,
    pub(crate) reg_id: u32,
}

impl EpollUringHandler {
    /// Construct EpollUringHandler with the given UringBearer
    pub fn with_bearer<C: core::fmt::Debug + Clone + OpCompletion>(
        bearer: &mut UringBearer<C>,
    ) -> Result<Self, EpollUringHandlerError> {
        // SAFETY: FFI no-data in
        let epfd = unsafe { libc::epoll_create1(0) };
        if epfd == -1 {
            // SAFETY: ffi no-data
            let errno = unsafe { libc::__errno_location() };
            return Err(EpollUringHandlerError::EpollCreate1(format!(
                "errno: {:?}",
                errno
            )));
        }

        let reg_epfd = RegisteredFd {
            kind: FdKind::EpollCtl,
            raw_fd: epfd,
        };

        let reg_id = bearer
            .add_registered_fd(reg_epfd)
            .map_err(EpollUringHandlerError::UringBearer)?;

        Ok(Self { epfd, reg_id })
    }
    /// The underlying epfd
    pub fn epfd(&self) -> RawFd {
        self.epfd
    }
    /// Fied registered Id for the underluing epfd
    pub fn reg_id(&self) -> u32 {
        self.reg_id
    }
}
