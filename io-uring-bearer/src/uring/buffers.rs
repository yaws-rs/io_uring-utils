//! ProvideBuffers OpCode API Surface

use crate::uring::UringBearerError;
use crate::Completion;

use crate::UringBearer;

use io_uring_owner::Owner;
use slabbable::Slabbable;

use std::num::NonZero;

use crate::slab::buffer::{TakenImmutableBuffer, TakenMutableBuffer};
use io_uring_opcode::OpCompletion;

impl<C: core::fmt::Debug + Clone + OpCompletion> UringBearer<C> {
    /// Internal API for use of Recv handing a single buffer.
    ///
    /// # Limitation
    ///
    /// Only take num_bufs == 1 buffers as the underlying type is not designed for splitting it up.
    pub(crate) fn take_one_mutable_buffer(
        &mut self,
        buf_idx: usize,
    ) -> Result<TakenMutableBuffer, UringBearerError> {
        let buf_ref = match self.bufs.slot_get_mut(buf_idx) {
            Ok(Some(buf)) => buf,
            Ok(None) => return Err(UringBearerError::BufferNotExist(buf_idx)),
            Err(e) => return Err(UringBearerError::Slabbable(e)),
        };
        Ok(
            crate::slab::buffer::take_one_mutable_buffer_raw(buf_idx, buf_ref)
                .map_err(UringBearerError::BufferTake)?,
        )
    }
    /// Internal API for use of Send/Zc handing out a single buffer.
    ///
    /// # Limitation
    ///
    /// Only take num_bufs == 1 buffers as the underlying type is not designed for splitting it up.
    pub(crate) fn take_one_immutable_buffer(
        &mut self,
        buf_idx: usize,
        buf_kernel_idx: u16,
    ) -> Result<TakenImmutableBuffer, UringBearerError> {
        let buf_ref = match self.bufs.slot_get_mut(buf_idx) {
            Ok(Some(buf)) => buf,
            Ok(None) => return Err(UringBearerError::BufferNotExist(buf_idx)),
            Err(e) => return Err(UringBearerError::Slabbable(e)),
        };
        Ok(
            crate::slab::buffer::take_one_immutable_buffer_raw(buf_idx, buf_kernel_idx, buf_ref)
                .map_err(UringBearerError::BufferTake)?,
        )
    }
}

impl<C: core::fmt::Debug + Clone + OpCompletion> UringBearer<C> {
    /// View a selected buffer unsafely where buf_idx is the created buffers index,
    /// buf_select is a buffer index within buffers created.
    ///
    /// # Safety
    ///
    /// We are not checking whether the kernel owns mutable reference to selefcted buffer.
    /// User must ensure kernel has no mutable reference into the selected buffer.
    ///
    /// # Panic
    ///
    /// expected_len must be > 0 and must be within the bounds of created buffer length.
    pub unsafe fn view_buffer_select_slice(
        &self,
        buf_idx: usize,
        buf_select: u16,
        expected_len: i32,
    ) -> Result<&[u8], UringBearerError> {
        assert!(expected_len > 0);
        let bufs = match self.bufs.slot_get_ref(buf_idx) {
            Ok(Some(itm)) => itm,
            Ok(None) => return Err(UringBearerError::BufferNotExist(buf_idx)),
            Err(e) => return Err(UringBearerError::Slabbable(e)),
        };
        assert!(expected_len <= bufs.len_per_buf());
        if bufs.num_bufs() < buf_select {
            return Err(UringBearerError::BufferSelectedNotExist(buf_select));
        }
        let all_bufs_ref = unsafe { bufs.all_bufs() };

        let mut chunks = all_bufs_ref.chunks_exact(bufs.len_per_buf() as usize);

        match chunks.nth(buf_select as usize) {
            Some(ret) => Ok(ret),
            None => Err(UringBearerError::BufferSelectedNotExist(buf_select)),
        }
    }
    /// Allocate new buffer-set and return it's created index.
    pub fn create_buffers(
        &mut self,
        num_bufs: NonZero<u16>,
        len_per_buf: i32,
    ) -> Result<usize, UringBearerError> {
        if len_per_buf <= 0 {
            return Err(UringBearerError::InvalidParameterI32(
                "UringBearer::create_buffers",
                "len_per_buf <= 0",
                len_per_buf,
            ));
        }
        self.bufs
            .take_next_with(crate::slab::buffer::construct_buffer(
                Owner::Created,
                len_per_buf,
                num_bufs.get(),
            ))
            .map_err(UringBearerError::Slabbable)
    }
    /// Destroy earlier created buffer-set by it's index.
    pub fn destroy_buffers(&mut self, id: usize) -> Result<(), UringBearerError> {
        match self.bufs.slot_get_ref(id) {
            Ok(Some(itm)) => match itm.owner() {
                Owner::Kernel => Err(UringBearerError::BufferNoOwnership(id)),
                _ => {
                    self.bufs
                        .mark_for_reuse(id)
                        .map_err(UringBearerError::Slabbable)?;
                    Ok(())
                }
            },
            Ok(None) => Err(UringBearerError::BufferNotExist(id)),
            Err(e) => Err(UringBearerError::Slabbable(e)),
        }
    }
    /// Provide earlier created buffer-set by it's index to Kernel
    pub fn provide_buffers(
        &mut self,
        created_buf_idx: usize,
        bgid: u16,
        bid: u16,
    ) -> Result<usize, UringBearerError> {
        let bufs_rec_ref = match self.bufs.slot_get_mut(created_buf_idx) {
            Err(e) => return Err(UringBearerError::Slabbable(e)),
            Ok(Some(ret)) => match ret.owner() {
                Owner::Kernel => return Err(UringBearerError::BufferNoOwnership(created_buf_idx)),
                _ => ret,
            },
            Ok(None) => return Err(UringBearerError::BufferNoOwnership(created_buf_idx)),
        };

        let key = self
            .fd_slab
            .take_next_with(Completion::ProvideBuffers(
                crate::slab::buffer::provide_buffer_rec(bgid, bid, bufs_rec_ref),
            ))
            .map_err(UringBearerError::Slabbable)?;
        let completion_rec = self
            .fd_slab
            .slot_get_ref(key)
            .map_err(UringBearerError::Slabbable)?;

        let iou = &mut self.io_uring;
        let mut s_queue = iou.submission();

        let submission = match completion_rec {
            Some(Completion::ProvideBuffers(provide_buffers_rec)) => {
                crate::slab::buffer::entry(provide_buffers_rec).user_data(key as u64)
            }
            _ => {
                return Err(UringBearerError::SlabBugSetGet(
                    "ProvideBuffers not found after set?",
                ))
            }
        };
        // SAFETY: We are backing the buffer & submission in the Slabbable stores. BufferRec buffer must not move
        // from the referred address nor otherwise manipulated or invalidated until the ownership passes back to userspace
        // or when the buffer/s are confirmed removed via RemoveBuffers otherwise.
        match unsafe { s_queue.push(&submission) } {
            Ok(_) => {
                bufs_rec_ref.force_owner_kernel();
                Ok(key)
            }
            Err(_) => Err(UringBearerError::SubmissionPush),
        }
    }
}
