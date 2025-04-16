//! Connect Record

use core::pin::Pin;

use crate::error::ConnectError;

use io_uring_opcode::OpExtConnect;
use io_uring_opcode::{OpCode, OpCompletion, OpError};
use io_uring_owner::Owner;

use ysockaddr::YSockAddrC;

/// Connect Record
#[derive(Clone, Debug)]
pub struct Connect {
    /// Current owner of the record
    owner: Owner,
    /// Related filehandle
    fixed_fd: u32,
    ysaddr: YSockAddrC,
}

impl Connect {
    /// Construct a new Connect
    pub fn with_ysockaddr_c(fixed_fd: u32, ysaddr: YSockAddrC) -> Result<Self, ConnectError> {
        Ok(Connect {
            owner: Owner::Created,
            fixed_fd,
            ysaddr,
        })
    }
}

impl OpCompletion for Connect {
    type Error = OpError;
    fn entry(&self) -> io_uring::squeue::Entry {
        let (saddr, slen) = self.ysaddr.as_c_sockaddr_len();

        println!("OpCompletion entry() saddr = {:p}, slen = {}", saddr, slen);
        
        io_uring::opcode::Connect::new(
            io_uring::types::Fixed(self.fixed_fd),
            saddr,
            slen)
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

impl OpCode<Connect> for Connect {
    fn submission(self) -> Result<Connect, OpError> {
        Ok(self)
    }
    fn completion(&mut self, _: Pin<&mut Connect>) -> Result<(), OpError> {
        todo!()
    }
}

impl OpExtConnect for Connect {
    /// Underlying Fixed Fd
    fn fixed_fd(&self) -> u32 {
        self.fixed_fd
    }
    /// Underlying YSockAddrC
    fn ysaddr(&self) -> &YSockAddrC {
        &self.ysaddr
    }
}
