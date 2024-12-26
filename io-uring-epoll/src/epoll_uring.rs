use crate::HandledFd;

use io_uring::opcode::EpollCtl;
use io_uring::IoUring;

use std::os::fd::RawFd;

#[derive(Debug)]
pub enum FdKind {
    /// Epoll ctl handle
    EpollCtl,
    /// EPoll Event Handle
    EpollEvent(HandledFd),
    /// Acceptor handle
    Acceptor,
    /// Manual handle
    Manual,
}

#[derive(Debug)]
pub struct RegisteredFd {
    kind: FdKind,
    raw_fd: RawFd,
}

#[derive(Debug)]
pub enum Completion {
    //    EpollEvent(libc::epoll_event),
    EpollEvent(epoll_ctl::EpollRec),
    Accept(accept::AcceptRec),
}

/// What to do with the submission record upon handling completion.
/// Used within handle_completions Fn Return
#[derive(Clone, Debug, PartialEq)]
pub enum SubmissionRecordStatus {
    /// Retain the original submsision record when it is needed to be retained.
    /// For example EpollCtl original Userdata must be retained in multishot mode.
    /// Downside is that care must be taken to clean up the associated sunmission record.
    Retain,
    /// Forget the associated submission record
    /// For example Accept original record can be deleted upon compleiton after read.
    /// Typically a new Accept submission is pushed without re-using any existing.    
    Forget,
}

/// Manage the io_uring Submission and Completion Queues
/// related to EpollCtrl opcode in io_uring.
/// See io_epoll(7):
/// <https://man7.org/linux/man-pages/man7/epoll.7.html>
pub struct EpollUringHandler {
    /// EpollCtl FD
    pub(crate) epfd: RawFd,
    /// io_uring Managed instance
    pub(crate) io_uring: IoUring<io_uring::squeue::Entry, io_uring::cqueue::Entry>,
    /// Completion events awaited
    pub(crate) fd_slab: slab::Slab<(usize, Completion)>,
    /// Registred Fds with io_uring
    pub(crate) fd_register: slab::Slab<(usize, RegisteredFd)>,
}

impl core::fmt::Debug for EpollUringHandler {
    fn fmt(&self, _: &mut core::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        Ok(())
    }
}

impl PartialEq for EpollUringHandler {
    fn eq(&self, _: &EpollUringHandler) -> bool {
        todo!()
    }
}

impl EpollUringHandler {
    /// Create a new handler with new io-uring::IoUring
    ///
    /// capacity must be power of two as per io-uring documentation
    /// ```rust
    /// use io_uring_epoll::EpollUringHandler;
    ///
    /// EpollUringHandler::new(10).expect("Unable to create EPoll Handler");
    /// ```    
    pub fn new(capacity: u32) -> Result<Self, EpollUringHandlerError> {
        let iou: IoUring<io_uring::squeue::Entry, io_uring::cqueue::Entry> = IoUring::builder()
            .build(capacity)
            .map_err(|e| EpollUringHandlerError::IoUringCreate(e.to_string()))?;

        Self::from_io_uring(iou)
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
    ///         .build(100)
    ///         .expect("Unable to build IoUring");
    ///
    /// EpollUringHandler::from_io_uring(iou).expect("Unable to create from io_uring Builder");
    /// ```    
    pub fn from_io_uring(
        iou: IoUring<io_uring::squeue::Entry, io_uring::cqueue::Entry>,
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

        Ok(Self {
            epfd,
            io_uring: iou,
            fd_slab: slab::Slab::new(),
            fd_register,
        })
    }
    /// epfd
    pub fn epfd(&self) -> RawFd {
        self.epfd
    }
    pub fn register_acceptor(&mut self, fd: RawFd) -> Result<usize, EpollUringHandlerError> {
        let entry = self.fd_register.vacant_entry();
        let key = entry.key();
        self.fd_register.insert((
            key,
            RegisteredFd {
                kind: FdKind::Acceptor,
                raw_fd: fd,
            },
        ));
        Ok(key)
    }
    /// Push register for handles
    /// Note kernel may mangle this table but generally adding entries
    /// Make sure the queues are empty before calling this
    pub fn commit_registered_handles(&mut self) -> Result<(), EpollUringHandlerError> {
        let iou = &mut self.io_uring;
        let submitter = iou.submitter();

        let mut vec_i32: Vec<RawFd> = Vec::with_capacity(self.fd_register.capacity());
        for idx in 0..self.fd_register.capacity() {
            if let Some((_, itm)) = self.fd_register.get(idx) {
                vec_i32.push(itm.raw_fd);
            } else {
                vec_i32.push(-1);
            }
        }

        let slice_i32: &[i32] = &vec_i32[0..];

        submitter.register_files(slice_i32);

        Ok(())
    }
    /// Spin the completions ring
    pub fn handle_completions<F, U>(
        &mut self,
        user: &mut U,
        func: F,
    ) -> Result<(), EpollUringHandlerError>
    where
        F: Fn(&mut U, &io_uring::cqueue::Entry, &Completion) -> SubmissionRecordStatus,
    {
        let iou = &mut self.io_uring;
        let mut c_queue = iou.completion();
        while let Some(item) = c_queue.next() {
            let key = item.user_data();
            let a_rec_t = self.fd_slab.get(key as usize);

            if let Some((_k, ref completed_rec)) = a_rec_t {
                let rec_status = func(user, &item, completed_rec);
                if rec_status == SubmissionRecordStatus::Forget {
                    self.fd_slab.remove(key as usize);
                }
            }
        }
        Ok(())
    }
    /// Borrow the underlying io-uring::IoUring instance
    pub fn io_uring(&mut self) -> &mut IoUring<io_uring::squeue::Entry, io_uring::cqueue::Entry> {
        &mut self.io_uring
    }
    /// This calls the underlying io_uring::IoUring::submit submitting all the staged commits
    pub fn submit(&self) -> Result<usize, EpollUringHandlerError> {
        self.io_uring
            .submit()
            .map_err(|e| EpollUringHandlerError::Submission(e.to_string()))
    }
    /// Same as submit but using io_uring::IoUring::submit_and_wait
    pub fn submit_and_wait(&self, want: usize) -> Result<usize, EpollUringHandlerError> {
        self.io_uring
            .submit_and_wait(want)
            .map_err(|e| EpollUringHandlerError::Submission(e.to_string()))
    }
    /// Add Accept for a IPv4 TCP Listener
    ///
    /// # Safety
    ///
    /// Use of a `fd` that is not a valid IPv4 TCP Listener is undefined behaviour.
    pub unsafe fn add_accept_ipv4(&mut self, fd: RawFd) -> Result<(), EpollUringHandlerError> {
        self.add_accept(fd, false)
    }
    /// Add Accept for a IPv6 TCP Listener
    ///
    /// # Safety
    ///
    /// Use of a `fd` that is not a valid IPv6 TCP Listener is undefined behaviour.
    pub unsafe fn add_accept_ipv6(&mut self, fd: RawFd) -> Result<(), EpollUringHandlerError> {
        self.add_accept(fd, true)
    }
    pub(crate) unsafe fn add_accept(
        &mut self,
        fd: RawFd,
        v6: bool,
    ) -> Result<(), EpollUringHandlerError> {
        let iou = &mut self.io_uring;
        let mut s_queue = iou.submission();

        let entry = self.fd_slab.vacant_entry();
        let key = entry.key();

        let _k = match v6 {
            true => entry.insert((key, Completion::Accept(accept::init_accept_rec6()))),
            false => entry.insert((key, Completion::Accept(accept::init_accept_rec4()))),
        };
        let a_rec_t = self.fd_slab.get(key);
        let dest_slot = None;
        let flags = libc::EFD_NONBLOCK & libc::EFD_CLOEXEC;

        match a_rec_t {
            Some((_k, Completion::Accept(a_rec_k))) => {
                let accept_rec =
                    accept::entry(fd, &a_rec_k, dest_slot, flags).user_data(key as u64);
                let accept = unsafe { s_queue.push(&accept_rec) };
            }
            _ => {
                return Err(EpollUringHandlerError::SlabBugSetGet(
                    "Accept not found after set?",
                ));
            }
        }

        Ok(())
    }
    /// Stage commit of all changed [`HandledFd`] resources to submission queue
    /// This will make all changes within the underlying io_uring::squeue::SubmissionQueue
    /// Use the [`Self::submit`] or [`Self::submit_and_wait`] to push the commits to kernel
    pub fn commit_fd(&mut self, handled_fd: &HandledFd) -> Result<(), EpollUringHandlerError> {
        let iou = &mut self.io_uring;
        let mut s_queue = iou.submission();

        let entry = self.fd_slab.vacant_entry();
        let key = entry.key();
        let _k = entry.insert((
            key,
            Completion::EpollEvent(epoll_ctl::event_rec(handled_fd, key as u64)),
        ));
        let e_rec_t = self.fd_slab.get(key);

        match e_rec_t {
            Some((k, Completion::EpollEvent(e_rec_k))) => {
                let add_rec = epoll_ctl::add(self.epfd, e_rec_k).user_data(key as u64);
                let add = unsafe { s_queue.push(&add_rec) };
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
