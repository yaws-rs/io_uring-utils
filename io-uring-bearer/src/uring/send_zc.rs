//! SendZc OpCode API Surface

use crate::uring::UringBearerError;
use crate::Completion;
use crate::SubmissionFlags;
use crate::UringBearer;

use crate::slab::SendZcRec;

use io_uring_opcode::OpCompletion;
use slabbable::Slabbable;

impl<C: core::fmt::Debug + Clone + OpCompletion> UringBearer<C> {
    /// Add SendZc pending Completion. The referenced buffers index must hold
    /// only one buffer and must be registered into kernel with a valid indexed
    /// buffer id.
    pub fn add_send_singlebuf(
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
}
