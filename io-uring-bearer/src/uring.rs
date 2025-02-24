//! Uring Handler

mod accept;
mod buffers;
mod futex;
mod recv;
mod register;
mod send_zc;

#[cfg(feature = "epoll")]
mod epoll_ctl;

use crate::error::UringBearerError;

use io_uring::IoUring;

use crate::completion::SubmissionRecordStatus;
use crate::fixed::FixedFdRegister;

use crate::slab::BuffersRec;
use crate::slab::FutexRec;
use crate::Completion;
use io_uring_owner::Owner;

use io_uring_opcode::{OpCode, OpCompletion};
use slabbable::Slabbable;
use slabbable_impl_selector::SelectedSlab;

use crate::BearerCapacityKind;
use capacity::Capacity;
use capacity::Setting as CapacitySetting;

/// Manage the io_uring Submission and Completion Queues
pub struct UringBearer<C> {
    /// io_uring Managed instance
    pub(crate) io_uring: IoUring<io_uring::squeue::Entry, io_uring::cqueue::Entry>,
    /// Completion events awaited
    pub(crate) fd_slab: SelectedSlab<Completion<C>>,
    /// Registred Fds with io_uring
    pub(crate) fd_register: FixedFdRegister,
    /// Allocated Buffers
    pub(crate) bufs: SelectedSlab<BuffersRec>,
    /// Futexes / Atomics
    pub(crate) futexes: SelectedSlab<FutexRec>,
}

impl<C: core::fmt::Debug + Clone + OpCompletion> UringBearer<C> {
    /// Create a new handler with new io-uring::IoUring
    ///
    /// capacity must be power of two as per io-uring documentation
    /// ```ignore
    /// use io_uring_epoll::UringBearer;
    ///
    /// UringBearer::new(16, 16, 16, 16, 16).expect("Unable to create EPoll Handler");
    /// ```    
    pub fn with_capacity<H: CapacitySetting<BearerCapacityKind>>(
        caps: Capacity<H, BearerCapacityKind>,
    ) -> Result<Self, UringBearerError> {
        let iou: IoUring<io_uring::squeue::Entry, io_uring::cqueue::Entry> = IoUring::builder()
            .build(caps.of_unbounded(&BearerCapacityKind::CoreQueue) as u32)
            .map_err(|e| UringBearerError::IoUringCreate(e.to_string()))?;

        Self::from_io_uring(iou, caps)
    }
    /// Create a new handler from an existing io-uring::IoUring builder
    /// To construct a custom IoUring see io-uring Builder:
    /// <https://docs.rs/io-uring/latest/io_uring/struct.Builder.html>
    ///
    /// Example:
    /// ```ignore
    /// use io_uring::IoUring;
    /// use io_uring_epoll::UringBearer;
    ///
    /// let mut iou: IoUring<io_uring::squeue::Entry, io_uring::cqueue::Entry>
    ///     = IoUring::builder()
    ///         .build(100)
    ///         .expect("Unable to build IoUring");
    ///
    /// UringBearer::from_io_uring(iou, 16, 16, 16, 16).expect("Unable to create from io_uring Builder");
    /// ```    
    pub fn from_io_uring<H: CapacitySetting<BearerCapacityKind>>(
        iou: IoUring<io_uring::squeue::Entry, io_uring::cqueue::Entry>,
        caps: Capacity<H, BearerCapacityKind>,
    ) -> Result<Self, UringBearerError> {
        Ok(Self {
            io_uring: iou,
            fd_slab: SelectedSlab::<Completion<C>>::with_fixed_capacity(
                caps.of_unbounded(&BearerCapacityKind::PendingCompletions),
            )
            .map_err(UringBearerError::Slabbable)?,
            fd_register: FixedFdRegister::with_fixed_capacity(
                caps.of_unbounded(&BearerCapacityKind::RegisteredFd) as u32,
            ),
            bufs: SelectedSlab::<BuffersRec>::with_fixed_capacity(
                caps.of_unbounded(&BearerCapacityKind::Buffers),
            )
            .map_err(UringBearerError::Slabbable)?,
            futexes: SelectedSlab::<FutexRec>::with_fixed_capacity(
                caps.of_unbounded(&BearerCapacityKind::Futexes),
            )
            .map_err(UringBearerError::Slabbable)?,
        })
    }
    /// Spin the completions ring with custom handling without touching the
    /// original submission record
    pub fn completions<F, U>(&mut self, user: &mut U, func: F) -> Result<(), UringBearerError>
    where
        F: Fn(&mut U, &io_uring::cqueue::Entry, &Completion<C>),
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
    ) -> Result<(), UringBearerError>
    where
        F: Fn(&mut U, &io_uring::cqueue::Entry, &Completion<C>) -> SubmissionRecordStatus,
    {
        let iou = &mut self.io_uring;
        let c_queue = iou.completion();
        for item in c_queue {
            let key = item.user_data();
            let a_rec_t = self
                .fd_slab
                .slot_get_ref(key as usize)
                .map_err(UringBearerError::Slabbable)?;

            if let Some(completed_rec) = a_rec_t {
                let rec_status = func(user, &item, completed_rec);
                if rec_status == SubmissionRecordStatus::Forget {
                    self.fd_slab
                        .mark_for_reuse(key as usize)
                        .map_err(UringBearerError::Slabbable)?;
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
    pub fn submit(&self) -> Result<usize, UringBearerError> {
        self.io_uring
            .submit()
            .map_err(|e| UringBearerError::Submission(e.to_string()))
    }
    /// Same as submit but using io_uring::IoUring::submit_and_wait
    pub fn submit_and_wait(&self, want: usize) -> Result<usize, UringBearerError> {
        self.io_uring
            .submit_and_wait(want)
            .map_err(|e| UringBearerError::Submission(e.to_string()))
    }
    /// Push a general Op implementing OpCode trait (see io-uring-opcode)
    pub fn push_op<Op: OpCode<C>>(&mut self, op: Op) -> Result<usize, UringBearerError> {
        let key = self
            .fd_slab
            .take_next_with(Completion::Op(op.submission()?))
            .map_err(UringBearerError::Slabbable)?;

        match self._push_to_completion(key) {
            Err(e) => Err(e),
            Ok(()) => Ok(key),
        }
    }
    /// Push a pending typed Completion directly
    pub fn push_op_typed(&mut self, op: Completion<C>) -> Result<usize, UringBearerError> {
        let key = self
            .fd_slab
            .take_next_with(op)
            .map_err(UringBearerError::Slabbable)?;

        match self._push_to_completion(key) {
            Err(e) => Err(e),
            Ok(()) => Ok(key),
        }
    }
    #[inline]
    pub(crate) fn _push_to_completion(&mut self, idx: usize) -> Result<(), UringBearerError> {
        let iou = &mut self.io_uring;
        let mut s_queue = iou.submission();

        let completion_rec = self
            .fd_slab
            .slot_get_mut(idx)
            .map_err(UringBearerError::Slabbable)?;

        let submission = match completion_rec {
            Some(completion) => {
                if completion.owner() == Owner::Kernel {
                    return Err(UringBearerError::InvalidOwnership(completion.owner(), idx));
                }
                completion.force_owner_kernel();
                completion.entry().user_data(idx as u64)
            }
            _ => return Err(UringBearerError::SlabBugSetGet("Submisison not found?")),
        };
        //                bufs_rec_ref.force_owner_kernel();
        // SAFETY: We are backing the buffer & submission in the Slabbable stores. BufferRec buffer must not move
        // from the referred address nor otherwise manipulated or invalidated until the ownership passes back to userspace
        // or when the buffer/s are confirmed removed via RemoveBuffers otherwise.
        match unsafe { s_queue.push(&submission) } {
            Ok(_) => Ok(()),
            Err(_) => Err(UringBearerError::SubmissionPush),
        }
    }
    // TODO: Rework - this should be generic and FdKind'ed
    #[inline]
    fn _fixed_fd_validate(&self, try_fixed_fd: u32) -> bool {
        if try_fixed_fd > self.fd_register.capacity() - 1 {
            return false;
        }
        match self.fd_register.get(try_fixed_fd) {
            Some((_, _itm)) => true,
            _ => false,
        }
    }
}
