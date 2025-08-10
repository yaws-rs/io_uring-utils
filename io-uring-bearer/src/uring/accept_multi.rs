//! Interaface for pushing AcceptMulti implementing OpExtAcceptMulti

use super::UringBearer;
use crate::error::UringBearerError;
use crate::Completion;
use crate::SubmissionFlags;
use io_uring_opcode::OpExtAcceptMulti;
use io_uring_opcode::{OpCode, OpCompletion};
use slabbable::Slabbable;

impl<C: core::fmt::Debug + Clone + OpCompletion> UringBearer<C> {
    /// Push a AcceptMulti implementing OpCode +  traits (see io-uring-opcode)
    pub fn push_accept_multi<Op>(
        &mut self,
        op: Op,
        flags: Option<SubmissionFlags>,
    ) -> Result<usize, UringBearerError>
    where
        Op: OpCode<C> + OpExtAcceptMulti,
    {
        let key = self
            .fd_slab
            .take_next_with(Completion::AcceptMulti(op.submission()?))
            .map_err(UringBearerError::Slabbable)?;

        match self._push_to_completion(key, flags) {
            Err(e) => Err(e),
            Ok(()) => Ok(key),
        }
    }
}
