//! Connect Record

use core::pin::Pin;

use crate::error::ConnectError;

use crate::HandledFd;
use crate::RawFd;

use io_uring_opcode::OpExtConnect;
use io_uring_opcode::{OpCode, OpCompletion, OpError};
use io_uring_owner::Owner;

use ysockaddr::YSockAddrC;

/// Connect Record
#[derive(Clone, Debug, PartialEq)]
pub struct Connect {
    /// Current owner of the record
    owner: Owner,
    /// Related filehandle
    fd: RawFd,
}

impl Connect {
    /// Construct a new Connect
    pub fn with_ysockaddr_c(ysaddr: YSockAddrC) -> Result<Self, ConnectError> {
        Ok(Connect {
            owner: Owner::Created,
            fd: create_new_fd,
        })
    }
}

impl OpCompletion for Connect {
    type Error = OpError;
    fn entry(&self) -> io_uring::squeue::Entry {
        io_uring::opcode::Connect::new(
            io_uring::types::Fixed(self.epfd as u32),
            io_uring::types::Fd(self.fd),
        )
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

impl OpCode<EpollCtl> for Connect {
    fn submission(self) -> Result<Connect, OpError> {
        Ok(self)
    }
    fn completion(&mut self, _: Pin<&mut Connect>) -> Result<(), OpError> {
        todo!()
    }
}

impl OpExtConnect for Connect {
    /// Underlying RawFd
    fn raw_fd(&self) -> RawFd {
        self.fd
    }
}
