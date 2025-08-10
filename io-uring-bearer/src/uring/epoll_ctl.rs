//! Interaface for pushing EpollCtl implementing OpExtEpollCtl

use super::UringBearer;
use crate::error::UringBearerError;
use crate::Completion;
use crate::SubmissionFlags;
use io_uring_opcode::OpExtEpollCtl;
use io_uring_opcode::{OpCode, OpCompletion};
use slabbable::Slabbable;

impl<C: core::fmt::Debug + Clone + OpCompletion> UringBearer<C> {
    /// Push an EpollCtl implementing OpCode +  traits (see io-uring-opcode)
    pub fn push_epoll_ctl<Op>(
        &mut self,
        op: Op,
        flags: Option<SubmissionFlags>,
    ) -> Result<usize, UringBearerError>
    where
        Op: OpCode<C> + OpExtEpollCtl,
    {
        let key = self
            .fd_slab
            .take_next_with(Completion::EpollCtl(op.submission()?))
            .map_err(UringBearerError::Slabbable)?;

        match self._push_to_completion(key, flags) {
            Err(e) => Err(e),
            Ok(()) => Ok(key),
        }
    }
}
