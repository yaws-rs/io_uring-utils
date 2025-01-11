//! Epoll Uring Handler

use crate::completion::Completion;
use crate::error::EpollUringHandlerError;
use crate::fd::{FdKind, HandledFd, RegisteredFd};
use crate::io_uring::IoUring;
use crate::RawFd;

// TODO: Refactor - This should come from UringHandler
use crate::error::UringHandlerError;

use crate::UringHandler;
use slabbable::Slabbable;

use io_uring::opcode::EpollCtl;

/// Manage the io_uring Submission and Completion Queues
/// related to EpollCtrl opcode in io_uring.
/// See io_epoll(7):
/// <https://man7.org/linux/man-pages/man7/epoll.7.html>
pub struct EpollUringHandler {
    /// EpollCtl FD
    pub(crate) epfd: RawFd,
    pub(crate) uring: UringHandler,
}

impl EpollUringHandler {
    /// Create a new EpollUringHandler.
    /// ```rust
    /// use io_uring_epoll::EpollUringHandler;
    ///
    /// EpollUringHandler::new(16, 16, 16).expect("Unable to create EpullUringHandler");
    /// ```    
    pub fn new(
        capacity: u32,
        fd_capacity: usize,
        req_capacity: usize,
    ) -> Result<Self, EpollUringHandlerError> {
        let iou: IoUring<io_uring::squeue::Entry, io_uring::cqueue::Entry> =
            IoUring::builder().build(capacity).map_err(|e| {
                EpollUringHandlerError::UringHandler(UringHandlerError::IoUringCreate(
                    e.to_string(),
                ))
            })?;
        Self::from_io_uring(iou, fd_capacity, req_capacity)
    }
    /// Create a new handler from an existing io-uring::IoUring builder
    /// To construct a custom IoUring see io-uring Builder:
    /// <https://docs.rs/io-uring/latest/io_uring/struct.Builder.html>
    ///
    /// Example:
    /// ```rust
    /// use io_uring::IoUring;
    /// use io_uring_epoll::EpollUringHandler;
    ///
    /// let mut iou: IoUring<io_uring::squeue::Entry, io_uring::cqueue::Entry>
    ///     = IoUring::builder()
    ///         .build(16)
    ///         .expect("Unable to build IoUring");
    ///
    /// EpollUringHandler::from_io_uring(iou, 16, 16).expect("Unable to create from io_uring Builder");
    /// ```    
    pub fn from_io_uring(
        iou: IoUring<io_uring::squeue::Entry, io_uring::cqueue::Entry>,
        fd_capacity: usize,
        req_capacity: usize,
    ) -> Result<Self, EpollUringHandlerError> {
        let mut epoll_probe = io_uring::Probe::new();
        iou.submitter()
            .register_probe(&mut epoll_probe)
            .map_err(|e| EpollUringHandlerError::Probing(e.to_string()))?;

        #[allow(clippy::bool_comparison)]
        if epoll_probe.is_supported(EpollCtl::CODE) == false {
            return Err(EpollUringHandlerError::NotSupported);
        }

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

        let mut fd_register = slab::Slab::new();
        fd_register.insert((
            0,
            RegisteredFd {
                kind: FdKind::EpollCtl,
                raw_fd: epfd,
            },
        ));

        let uring = UringHandler::from_io_uring(iou, fd_capacity, req_capacity)
            .map_err(EpollUringHandlerError::UringHandler)?;

        Ok(Self { epfd, uring })
    }
    /// The underlying epfd
    pub fn epfd(&self) -> RawFd {
        self.epfd
    }
    /// The underlying UringHandler instance
    pub fn uring_handler(&mut self) -> &mut UringHandler {
        &mut self.uring
    }
    /// Borrow the underlying io-uring::IoUring instance
    pub fn io_uring(&mut self) -> &mut IoUring<io_uring::squeue::Entry, io_uring::cqueue::Entry> {
        self.uring.io_uring()
    }
    /// This calls the underlying io_uring::IoUring::submit submitting all the staged commits
    pub fn submit(&self) -> Result<usize, EpollUringHandlerError> {
        self.uring.submit().map_err(|e| {
            EpollUringHandlerError::UringHandler(UringHandlerError::Submission(e.to_string()))
        })
    }
    /// Same as submit but using io_uring::IoUring::submit_and_wait
    pub fn submit_and_wait(&self, want: usize) -> Result<usize, EpollUringHandlerError> {
        self.uring.submit_and_wait(want).map_err(|e| {
            EpollUringHandlerError::UringHandler(UringHandlerError::Submission(e.to_string()))
        })
    }
    /// Stage commit of all changed [`HandledFd`] resources to submission queue
    /// This will make all changes within the underlying io_uring::squeue::SubmissionQueue
    /// Use the [`Self::submit`] or [`Self::submit_and_wait`] to push the commits to kernel
    pub fn commit_fd(&mut self, handled_fd: &HandledFd) -> Result<(), EpollUringHandlerError> {
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
    }
}
