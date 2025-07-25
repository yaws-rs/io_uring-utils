//! Interaface for pushing Connect implementing OpExtConnect

use super::UringBearer;
use crate::error::UringBearerError;
use crate::Completion;
use io_uring_opcode::OpExtConnect;
use io_uring_opcode::{OpCode, OpCompletion};
use slabbable::Slabbable;

impl<C: core::fmt::Debug + Clone + OpCompletion> UringBearer<C> {
    /// Push a Connect implementing OpCode +  traits (see io-uring-opcode)
    pub fn push_connect<Op>(&mut self, op: Op) -> Result<usize, UringBearerError>
    where
        Op: OpCode<C> + OpExtConnect,
    {
        let key = self
            .fd_slab
            .take_next_with(Completion::Connect(op.submission()?))
            .map_err(UringBearerError::Slabbable)?;

        match self._push_to_completion(key) {
            Err(e) => Err(e),
            Ok(()) => Ok(key),
        }
    }
}
