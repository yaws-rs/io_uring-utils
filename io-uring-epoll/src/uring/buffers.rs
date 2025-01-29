//! ProvideBuffers OpCode API Surface

use crate::uring::UringHandlerError;
use crate::Completion;
use crate::Owner;
use crate::UringHandler;

use slabbable::Slabbable;

use std::num::NonZero;

use crate::slab::buffer::TakenBuffer;

impl UringHandler {
    /// Internal API for use of Recv handing out the buffer.
    ///
    /// # Limitation
    ///
    /// Only take num_bufs == 1 buffers as the underlying type is not designed for splitting it up.
    // TODO: Should this be pub?
    pub(crate) fn take_buffer(&mut self, buf_idx: usize) -> Result<TakenBuffer, UringHandlerError> {
        let buf_ref = match self.bufs.slot_get_mut(buf_idx) {
            Ok(Some(buf)) => buf,
            Ok(None) => return Err(UringHandlerError::BufferNotExist(buf_idx)),
            Err(e) => return Err(UringHandlerError::Slabbable(e)),
        };
        Ok(crate::slab::buffer::take_one_buffer_raw(buf_idx, buf_ref)
            .map_err(UringHandlerError::BufferTake)?)
    }
}

impl UringHandler {
    /// Allocate new buffer-set and return it's created index.
    pub fn create_buffers(
        &mut self,
        num_bufs: NonZero<u16>,
        len_per_buf: i32,
    ) -> Result<usize, UringHandlerError> {
        if len_per_buf <= 0 {
            return Err(UringHandlerError::InvalidParameterI32(
                "UringHandler::create_buffers",
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
            .map_err(UringHandlerError::Slabbable)
    }
    /// Destroy earlier created buffer-set by it's index.
    pub fn destroy_buffers(&mut self, id: usize) -> Result<(), UringHandlerError> {
        match self.bufs.slot_get_ref(id) {
            Ok(Some(itm)) => match itm.owner() {
                Owner::Kernel => Err(UringHandlerError::BufferNoOwnership(id)),
                _ => {
                    self.bufs
                        .mark_for_reuse(id)
                        .map_err(UringHandlerError::Slabbable)?;
                    Ok(())
                }
            },
            Ok(None) => Err(UringHandlerError::BufferNotExist(id)),
            Err(e) => Err(UringHandlerError::Slabbable(e)),
        }
    }
    /// Provide earlier created buffer-set by it's index to Kernel
    pub fn provide_buffers(
        &mut self,
        created_buf_idx: usize,
        bgid: u16,
        bid: u16,
    ) -> Result<usize, UringHandlerError> {
        let bufs_rec_ref = match self.bufs.slot_get_mut(created_buf_idx) {
            Err(e) => return Err(UringHandlerError::Slabbable(e)),
            Ok(Some(ret)) => match ret.owner() {
                Owner::Kernel => return Err(UringHandlerError::BufferNoOwnership(created_buf_idx)),
                _ => ret,
            },
            Ok(None) => return Err(UringHandlerError::BufferNoOwnership(created_buf_idx)),
        };
        let key = self
            .fd_slab
            .take_next_with(Completion::ProvideBuffers(
                crate::slab::buffer::provide_buffer_rec(bgid, bid, bufs_rec_ref),
            ))
            .map_err(UringHandlerError::Slabbable)?;
        let completion_rec = self
            .fd_slab
            .slot_get_ref(key)
            .map_err(UringHandlerError::Slabbable)?;

        let iou = &mut self.io_uring;
        let mut s_queue = iou.submission();

        let submission = match completion_rec {
            Some(Completion::ProvideBuffers(provide_buffers_rec)) => {
                crate::slab::buffer::entry(provide_buffers_rec).user_data(key as u64)
            }
            _ => {
                return Err(UringHandlerError::SlabBugSetGet(
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
            Err(_) => Err(UringHandlerError::SubmissionPush),
        }
    }
}
