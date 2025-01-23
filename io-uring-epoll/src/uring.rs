//! Uring Handler

mod accept;
mod buffers;
mod futex;

use crate::error::UringHandlerError;

use io_uring::IoUring;

use std::os::fd::RawFd;

use crate::completion::SubmissionRecordStatus;
use crate::fd::RegisteredFd;

use crate::slab::BuffersRec;
use crate::slab::FutexRec;
use crate::Completion;

use slab::Slab;
use slabbable::Slabbable;
use slabbable_impl_selector::SelectedSlab;

/// Manage the io_uring Submission and Completion Queues
pub struct UringHandler {
    /// io_uring Managed instance
    pub(crate) io_uring: IoUring<io_uring::squeue::Entry, io_uring::cqueue::Entry>,
    /// Completion events awaited
    pub(crate) fd_slab: SelectedSlab<Completion>,
    /// Registred Fds with io_uring
    pub(crate) fd_register: Slab<(usize, RegisteredFd)>,
    /// Allocated Buffers
    pub(crate) bufs: SelectedSlab<BuffersRec>,
    /// Futexes / Atomics
    pub(crate) futexes: SelectedSlab<FutexRec>,
}

impl UringHandler {
    /// Create a new handler with new io-uring::IoUring
    ///
    /// capacity must be power of two as per io-uring documentation
    /// ```rust
    /// use io_uring_epoll::UringHandler;
    ///
    /// UringHandler::new(16, 16, 16, 16, 16).expect("Unable to create EPoll Handler");
    /// ```    
    pub fn new(
        q_capacity: u32,
        fd_capacity: usize,
        req_capacity: usize,
        buf_capacity: usize,
        fut_capacity: usize,
    ) -> Result<Self, UringHandlerError> {
        let iou: IoUring<io_uring::squeue::Entry, io_uring::cqueue::Entry> = IoUring::builder()
            .build(q_capacity)
            .map_err(|e| UringHandlerError::IoUringCreate(e.to_string()))?;

        Self::from_io_uring(iou, fd_capacity, req_capacity, buf_capacity, fut_capacity)
    }
    /// Create a new handler from an existing io-uring::IoUring builder
    /// To construct a custom IoUring see io-uring Builder:
    /// <https://docs.rs/io-uring/latest/io_uring/struct.Builder.html>
    ///
    /// Example:
    /// ```rust
    /// use io_uring::IoUring;
    /// use io_uring_epoll::UringHandler;
    ///
    /// let mut iou: IoUring<io_uring::squeue::Entry, io_uring::cqueue::Entry>
    ///     = IoUring::builder()
    ///         .build(100)
    ///         .expect("Unable to build IoUring");
    ///
    /// UringHandler::from_io_uring(iou, 16, 16, 16, 16).expect("Unable to create from io_uring Builder");
    /// ```    
    pub fn from_io_uring(
        iou: IoUring<io_uring::squeue::Entry, io_uring::cqueue::Entry>,
        fd_capacity: usize,
        req_capacity: usize,
        buf_capacity: usize,
        fut_capacity: usize,
    ) -> Result<Self, UringHandlerError> {
        Ok(Self {
            io_uring: iou,
            fd_slab: SelectedSlab::<Completion>::with_fixed_capacity(req_capacity)
                .map_err(UringHandlerError::Slabbable)?,
            fd_register: Slab::with_capacity(fd_capacity),
            bufs: SelectedSlab::<BuffersRec>::with_fixed_capacity(buf_capacity)
                .map_err(UringHandlerError::Slabbable)?,
            futexes: SelectedSlab::<FutexRec>::with_fixed_capacity(fut_capacity)
                .map_err(UringHandlerError::Slabbable)?,
        })
    }
    /// Push register for handles
    /// Note kernel may mangle this table but generally adding entries to reserved -1 ones.
    /// Generally make sure the queues are / can go empty before calling this
    pub fn commit_registered_handles(&mut self) -> Result<(), UringHandlerError> {
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

        submitter
            .register_files(slice_i32)
            .map_err(|e| UringHandlerError::RegisterHandles(e.to_string()))?;

        Ok(())
    }
    /// Spin the completions ring with custom handling without touching the
    /// original submission record
    pub fn completions<F, U>(&mut self, user: &mut U, func: F) -> Result<(), UringHandlerError>
    where
        F: Fn(&mut U, &io_uring::cqueue::Entry, &Completion),
    {
        // SAFETY: We Retain the original submission record and don't move it.
        unsafe {
            self.handle_completions(user, |u, e, rec| {
                func(u, e, rec);
                SubmissionRecordStatus::Retain
            })
        }
    }
    /// Spin the completions ring with custom handling on the original submission record.
    /// See [`completions`] for the safe variant that requires retaining
    /// the original submission record.
    ///
    /// # Safety
    ///
    /// Record must be retained if it is to be used after handling it
    /// e.g. EpollEvent record must be retained if it will trigger again given
    /// kernel still refers to it upon it triggering where as upon deleting handle
    /// from EpollCtl it can be only deleted after it has been confirmed as deleted.
    pub unsafe fn handle_completions<F, U>(
        &mut self,
        user: &mut U,
        func: F,
    ) -> Result<(), UringHandlerError>
    where
        F: Fn(&mut U, &io_uring::cqueue::Entry, &Completion) -> SubmissionRecordStatus,
    {
        let iou = &mut self.io_uring;
        let c_queue = iou.completion();
        for item in c_queue {
            let key = item.user_data();
            let a_rec_t = self
                .fd_slab
                .slot_get_ref(key as usize)
                .map_err(UringHandlerError::Slabbable)?;

            if let Some(completed_rec) = a_rec_t {
                let rec_status = func(user, &item, completed_rec);
                if rec_status == SubmissionRecordStatus::Forget {
                    self.fd_slab
                        .mark_for_reuse(key as usize)
                        .map_err(UringHandlerError::Slabbable)?;
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
    pub fn submit(&self) -> Result<usize, UringHandlerError> {
        self.io_uring
            .submit()
            .map_err(|e| UringHandlerError::Submission(e.to_string()))
    }
    /// Same as submit but using io_uring::IoUring::submit_and_wait
    pub fn submit_and_wait(&self, want: usize) -> Result<usize, UringHandlerError> {
        self.io_uring
            .submit_and_wait(want)
            .map_err(|e| UringHandlerError::Submission(e.to_string()))
    }
}
