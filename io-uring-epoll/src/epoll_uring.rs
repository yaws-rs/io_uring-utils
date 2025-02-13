//! EpollCtl OpCode Handler

//use crate::completion::Completion;
use crate::error::EpollUringHandlerError;
use crate::RawFd;

use io_uring_bearer::UringBearer;

use io_uring_fd::{FdKind, RegisteredFd};
//use io_uring::opcode::EpollCtl;

use crate::HandledFd;

use io_uring_opcode::{OpCode, OpCompletion};

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
}

//    pub fn submission(&mut self, handled_fd: &HandledFd) -> Result<UringSubmission, EpollUringHandlerError> {
//
/*
let iou = &mut self.uring.io_uring;
let mut s_queue = iou.submission();

let reserved =
    self.uring.fd_slab.reserve_next().map_err(|e| {
        EpollUringHandlerError::UringHandler(UringHandlerError::Slabbable(e))
    })?;
let udata = reserved.id();

let key = self
    .uring
    .fd_slab
    .take_reserved_with(
        reserved,
        Completion::EpollEvent(crate::slab::epoll_ctl::event_rec(handled_fd, udata as u64)),
    )
    .map_err(|e| EpollUringHandlerError::UringHandler(UringHandlerError::Slabbable(e)))?;
let e_rec_t =
    self.uring.fd_slab.slot_get_ref(key).map_err(|e| {
        EpollUringHandlerError::UringHandler(UringHandlerError::Slabbable(e))
    })?;

match e_rec_t {
    Some(Completion::EpollEvent(e_rec_k)) => {
        let add_rec = crate::slab::epoll_ctl::add(self.epfd, e_rec_k).user_data(key as u64);
        let _add = unsafe { s_queue.push(&add_rec) };
    }
    _ => {
        return Err(EpollUringHandlerError::SlabBugSetGet(
            "EpollEvent not found after set?",
        ));
    }
}

Ok(())
*/
//    }
//}
