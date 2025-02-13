//! Futex OpCodes API Surface

use crate::uring::UringBearerError;
use crate::Completion;
use crate::Owner;
use crate::UringBearer;

use std::sync::atomic::AtomicU32;
use std::sync::Arc;

use crate::slab::futex::FutexRec;

use slabbable::Slabbable;

use io_uring_opcode::OpCompletion;

impl<C: core::fmt::Debug + Clone + OpCompletion> UringBearer<C> {
    /// UringBearer creates a new Futex Atomic and provides indexed key to it.
    pub fn create_futex_atomic(&mut self) -> Result<usize, UringBearerError> {
        self.futexes
            .take_next_with(crate::slab::futex::construct_futex())
            .map_err(UringBearerError::Slabbable)
    }
    /// User supplies pre-existing Atomic as a new Futex Atomic and UringBearer provides indexed key to it.
    ///
    /// # Safety
    ///
    /// Supplied Atomic must not move or be invalidated otherwise, including, during pending FutexWait completion.
    /// Other atomic restrictions such as only atomic operations must be upheld.
    pub unsafe fn supply_futex_atomic_raw(
        &mut self,
        f: *const u32,
    ) -> Result<usize, UringBearerError> {
        self.futexes
            .take_next_with(crate::slab::futex::construct_unsafe_futex(f))
            .map_err(UringBearerError::Slabbable)
    }
    /// Remove indexed futex that acts as no dependency for any pending completion.
    pub fn remove_futex_atomic(&mut self, futex_idx: usize) -> Result<(), UringBearerError> {
        match self.futexes.slot_get_ref(futex_idx) {
            Ok(Some(itm)) => match itm.owner() {
                Owner::Kernel => Err(UringBearerError::FutexNoOwnership(futex_idx)),
                _ => {
                    self.futexes
                        .mark_for_reuse(futex_idx)
                        .map_err(UringBearerError::Slabbable)?;
                    Ok(())
                }
            },
            Ok(None) => Err(UringBearerError::FutexNotExist(futex_idx)),
            Err(e) => Err(UringBearerError::Slabbable(e)),
        }
    }
    /// Get Atomic reference to indexed owned futex. All operations to it must be atomic.
    pub fn get_futex_arc(&self, futex_idx: usize) -> Result<Arc<AtomicU32>, UringBearerError> {
        match self.futexes.slot_get_ref(futex_idx) {
            Ok(Some(itm)) => match itm {
                FutexRec::Owned(ref o_futex) => Ok(o_futex.atom_arc()),
                FutexRec::UnsafeReferenced(_) => Err(UringBearerError::FutexNoOwnership(futex_idx)),
            },
            Ok(None) => Err(UringBearerError::FutexNotExist(futex_idx)),
            Err(e) => Err(UringBearerError::Slabbable(e)),
        }
    }
    /// Submit FutexWait on indexed atomic that was either created with create_futex_atomic or supplied via supply_futex_atomic
    pub fn add_futex_wait(
        &mut self,
        futex_idx: usize,
        bitset: u64,
        val: u64,
    ) -> Result<usize, UringBearerError> {
        let ftx_rec_ref = match self.futexes.slot_get_mut(futex_idx) {
            Ok(Some(itm)) => match itm.owner() {
                Owner::Kernel => return Err(UringBearerError::FutexNoOwnership(futex_idx)),
                _ => itm,
            },
            Ok(None) => return Err(UringBearerError::FutexNotExist(futex_idx)),
            Err(e) => return Err(UringBearerError::Slabbable(e)),
        };
        let key = self
            .fd_slab
            .take_next_with(Completion::FutexWait(crate::slab::futex::wait_futex_rec(
                bitset,
                val,
                ftx_rec_ref,
            )))
            .map_err(UringBearerError::Slabbable)?;
        let futex_wait_rec = self
            .fd_slab
            .slot_get_ref(key)
            .map_err(UringBearerError::Slabbable)?;

        let iou = &mut self.io_uring;
        let mut s_queue = iou.submission();

        let submission = match futex_wait_rec {
            Some(Completion::FutexWait(futex_wait_rec)) => {
                crate::slab::futex::entry(futex_wait_rec).user_data(key as u64)
            }
            _ => {
                return Err(UringBearerError::SlabBugSetGet(
                    "FutexWait not found after set?",
                ))
            }
        };
        // SAFETY: We don't allow move / invalidation of the safe owned atomic given other guarantees hold.
        match unsafe { s_queue.push(&submission) } {
            Ok(_) => {
                ftx_rec_ref.force_owner_kernel();
                Ok(key)
            }
            Err(_) => Err(UringBearerError::SubmissionPush),
        }
    }
}
