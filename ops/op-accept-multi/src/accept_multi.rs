//! AcceptMulti Record

use core::pin::Pin;

use crate::error::AcceptMultiError;

use io_uring_opcode::OpExtAcceptMulti;
use io_uring_opcode::{OpCode, OpCompletion, OpError};
use io_uring_owner::Owner;

/// AcceptMulti Record
#[derive(Clone, Debug)]
pub struct AcceptMulti {
    /// Current owner of the record
    owner: Owner,
    /// Related filehandle (of acceptor)
    fixed_fd: u32,
    /// Whether all accepted filehandles are fixed (mapped) to io_uring or regular.
    accept_fixed: bool,
}

impl AcceptMulti {
    /// Construct a new AcceptMulti resulting Regular filehandles accepted
    pub fn with_regular_fds(fixed_fd: u32) -> Result<Self, AcceptMultiError> {
        Ok(AcceptMulti {
            owner: Owner::Created,
            fixed_fd,
            accept_fixed: false,
        })
    }
    /// Construct a new AcceptMulti resulting Fixed filehandles accepted
    pub fn with_fixed_fds(fixed_fd: u32) -> Result<Self, AcceptMultiError> {
        Ok(AcceptMulti {
            owner: Owner::Created,
            fixed_fd,
            accept_fixed: true,
        })
    }
}

impl OpCompletion for AcceptMulti {
    type Error = OpError;
    fn entry(&self) -> io_uring::squeue::Entry {
        io_uring::opcode::AcceptMulti::new(io_uring::types::Fixed(self.fixed_fd))
            .allocate_file_index(self.accept_fixed)
            .build()
    }
    fn owner(&self) -> Owner {
        self.owner.clone()
    }
    fn force_owner_kernel(&mut self) -> bool {
        self.owner = Owner::Kernel;
        true
    }
}

impl OpCode<AcceptMulti> for AcceptMulti {
    fn submission(self) -> Result<AcceptMulti, OpError> {
        Ok(self)
    }
    fn completion(&mut self, _: Pin<&mut AcceptMulti>) -> Result<(), OpError> {
        todo!()
    }
}

impl OpExtAcceptMulti for AcceptMulti {
    /// Underlying Fixed Fd
    fn fixed_fd(&self) -> u32 {
        self.fixed_fd
    }
}
