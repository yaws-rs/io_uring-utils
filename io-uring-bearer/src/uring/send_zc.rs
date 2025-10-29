//! SendZc OpCode API Surface

use crate::uring::UringBearerError;
use crate::Completion;
use crate::SubmissionFlags;
use crate::UringBearer;

use crate::slab::SendZcRec;

use crate::slab::send_zc::DestTo;
use io_uring_opcode::OpCompletion;
use slabbable::Slabbable;

impl<C: core::fmt::Debug + Clone + OpCompletion> UringBearer<C> {
    /// Add SendZc pending Completion. The referenced buffers index must hold
    /// only one buffer and must be registered into kernel with a valid indexed
    /// buffer id.
    #[inline]
    pub fn add_send_zc_singlebuf(
        &mut self,
        fixed_fd: u32,
        buf_idx: usize,
        kernel_index: u16,
        flags: Option<SubmissionFlags>,
    ) -> Result<usize, UringBearerError> {
        let taken_buf = self.take_one_immutable_buffer(buf_idx, kernel_index)?;
        if !self._fixed_fd_validate(fixed_fd) {
            return Err(UringBearerError::FdNotRegistered(fixed_fd));
        }
        let key = self
            .fd_slab
            .take_next_with(Completion::SendZc(SendZcRec::with_fixed_buf(
                fixed_fd as u32,
                taken_buf,
            )))
            .map_err(UringBearerError::Slabbable)?;

        self._push_to_completion(key, flags)?;
        Ok(key)
    }
    /// Zero-Copy Send with the supplied raw buffer which not managed by the bearer.
    ///
    /// ## SAFETY
    /// The caller must ensure the raw buffer is valid the entirety of completion and usage and
    /// that the buffer is valid to entire length in addition to holding all the guarantees
    /// required for the fixed buffer use with SendZc.
    #[inline]
    pub unsafe fn add_send_zc_rawbuf(
        &mut self,
        fixed_fd: u32,
        raw_buf_ptr: *const u8,
        raw_buf_size: u32,
        to_addr: Option<DestTo>,
        flags: Option<SubmissionFlags>,
    ) -> Result<usize, UringBearerError> {
        let key = self
            .fd_slab
            .take_next_with(Completion::SendZc(SendZcRec::with_unsafe_rawbuf(
                fixed_fd,
                raw_buf_ptr,
                raw_buf_size,
                to_addr,
            )))
            .map_err(UringBearerError::Slabbable)?;
        self._push_to_completion(key, flags)?;

        Ok(key)
    }
}
