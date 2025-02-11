//! SendZc OpCode API Surface

use crate::uring::UringHandlerError;
use crate::Completion;
use crate::UringHandler;

use crate::slab::SendZcRec;

use slabbable::Slabbable;

impl UringHandler {
    /// Add SendZc pending Completion. The referenced buffers index must hold
    /// only one buffer and must be registered into kernel with a valid indexed
    /// buffer id.
    pub fn add_send_singlebuf(
        &mut self,
        fixed_fd: u32,
        buf_idx: usize,
        kernel_index: u16,
    ) -> Result<usize, UringHandlerError> {
        let taken_buf = self.take_one_immutable_buffer(buf_idx, kernel_index)?;
        if !self._fixed_fd_validate(fixed_fd) {
            return Err(UringHandlerError::FdNotRegistered(fixed_fd));
        }
        let key = self
            .fd_slab
            .take_next_with(Completion::SendZc(SendZcRec::with_fixed_buf(
                fixed_fd as u32,
                taken_buf,
            )))
            .map_err(UringHandlerError::Slabbable)?;

        self.push_completion(key)?;
        Ok(key)
    }
}
