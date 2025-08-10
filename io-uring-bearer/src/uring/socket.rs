//! Interaface for pushing Socket implementing OpExtSocket

use super::UringBearer;
use crate::error::UringBearerError;
use crate::Completion;
use crate::SubmissionFlags;
use io_uring_opcode::OpExtSocket;
use io_uring_opcode::{OpCode, OpCompletion};
use slabbable::Slabbable;

impl<C: core::fmt::Debug + Clone + OpCompletion> UringBearer<C> {
    /// Push a Socket implementing OpCode +  traits (see io-uring-opcode)
    pub fn push_socket<Op>(
        &mut self,
        op: Op,
        flags: Option<SubmissionFlags>,
    ) -> Result<usize, UringBearerError>
    where
        Op: OpCode<C> + OpExtSocket,
    {
        let key = self
            .fd_slab
            .take_next_with(Completion::Socket(op.submission()?))
            .map_err(UringBearerError::Slabbable)?;

        match self._push_to_completion(key, flags) {
            Err(e) => Err(e),
            Ok(()) => Ok(key),
        }
    }
}
