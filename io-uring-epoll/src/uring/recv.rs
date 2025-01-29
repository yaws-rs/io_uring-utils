//! Recv OpCodes API Surface

use crate::uring::UringHandlerError;
use crate::Completion;
use crate::UringHandler;

use crate::slab::{RecvMultiRec, RecvRec};

use slabbable::Slabbable;

impl UringHandler {
    // TODO: this should be generic and FdKind'ed
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
    /// Add Recv pending Completion
    pub fn add_recv(&mut self, fixed_fd: u32, buf_idx: usize) -> Result<usize, UringHandlerError> {
        let taken_buf = self.take_buffer(buf_idx)?;
        if !self._fixed_fd_validate(fixed_fd) {
            return Err(UringHandlerError::FdNotRegistered(fixed_fd));
        }
        let key = self
            .fd_slab
            .take_next_with(Completion::Recv(RecvRec::new(fixed_fd as u32, taken_buf)))
            .map_err(UringHandlerError::Slabbable)?;

        self.push_completion(key)?;
        Ok(key)
    }
    /// Add RecvMulti pending Completion
    pub fn add_recv_multi(
        &mut self,
        fixed_fd: u32,
        buf_group: u16,
    ) -> Result<usize, UringHandlerError> {
        if !self._fixed_fd_validate(fixed_fd) {
            return Err(UringHandlerError::FdNotRegistered(fixed_fd));
        }
        let key = self
            .fd_slab
            .take_next_with(Completion::RecvMulti(RecvMultiRec::new(
                fixed_fd as u32,
                buf_group,
            )))
            .map_err(UringHandlerError::Slabbable)?;

        self.push_completion(key)?;
        Ok(key)
    }
}
