//! SendZc Record

use core::pin::Pin;

use crate::error::SendZcError;

use io_uring_opcode::OpExtSendZc;
use io_uring_opcode::{OpCode, OpCompletion, OpError};
use io_uring_owner::Owner;

/// SendZc Record
#[derive(Clone, Debug)]
pub struct SendZc<U> {
    owner: Owner,
    fixed_fd: u32,
    buf_const_u8: *const u8,
    buf_size: u32,
    buf_ref: usize,
    #[allow(dead_code)]
    to_addr: Option<DestTo>,
}

impl SendZc<U> {
    ///
    pub fn with_regular_fds(fixed_fd: u32) -> Result<Self, AcceptMultiError> {
    }
}

impl OpCompletion for SendZc {
    type Error = OpError;
    fn entry(&self) -> io_uring::squeue::Entry {
        io_uring::opcode::SendZc::new(
            io_uring::types::Fixed(self.fixed_fd), self.buf_const_u8, self.buf_len)
            .buf_index(buf_index)
            .build()
        // todo dest_addr(), dest_addr_len(), zc_flags()
    }
    fn owner(&self) -> Owner {
        self.owner.clone()
    }
    fn force_owner_kernel(&mut self) -> bool {
        self.owner = Owner::Kernel;
        true
    }
}

impl OpCode<SendZc> for SendZc {
    fn submission(self) -> Result<SendZc, OpError> {
        Ok(self)
    }
    fn completion(&mut self, _: Pin<&mut SendZc>) -> Result<(), OpError> {
        todo!()
    }
}

impl OpExtSendZc for SendZc {
    /// Underlying Fixed Fd
    fn fixed_fd(&self) -> u32 {
        self.fixed_fd
    }
}
