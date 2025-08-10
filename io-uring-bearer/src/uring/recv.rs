//! Recv OpCodes API Surface

use crate::uring::UringBearerError;
use crate::Completion;
use crate::UringBearer;

use crate::slab::{RecvMultiRec, RecvRec};
use crate::SubmissionFlags;

use io_uring_opcode::OpCompletion;
use slabbable::Slabbable;

impl<C: core::fmt::Debug + Clone + OpCompletion> UringBearer<C> {
    /// Add Recv pending Completion. The referenced buffers index must hold only one buffer.
    pub fn add_recv(
        &mut self,
        fixed_fd: u32,
        buf_idx: usize,
        flags: Option<SubmissionFlags>,
    ) -> Result<usize, UringBearerError> {
        let taken_buf = self.take_one_mutable_buffer(buf_idx)?;
        if !self._fixed_fd_validate(fixed_fd) {
            return Err(UringBearerError::FdNotRegistered(fixed_fd));
        }
        let key = self
            .fd_slab
            .take_next_with(Completion::Recv(RecvRec::new(fixed_fd as u32, taken_buf)))
            .map_err(UringBearerError::Slabbable)?;

        self._push_to_completion(key, flags)?;
        Ok(key)
    }
    /// Add RecvMulti pending Completion
    pub fn add_recv_multi(
        &mut self,
        fixed_fd: u32,
        buf_group: u16,
        flags: Option<SubmissionFlags>,
    ) -> Result<usize, UringBearerError> {
        if !self._fixed_fd_validate(fixed_fd) {
            return Err(UringBearerError::FdNotRegistered(fixed_fd));
        }
        let key = self
            .fd_slab
            .take_next_with(Completion::RecvMulti(RecvMultiRec::new(
                fixed_fd as u32,
                buf_group,
            )))
            .map_err(UringBearerError::Slabbable)?;

        self._push_to_completion(key, flags)?;
        Ok(key)
    }
}
