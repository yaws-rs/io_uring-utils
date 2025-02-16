//! EpollCtl Record

use core::pin::Pin;

use crate::error::EpollCtlError;

use crate::HandledFd;
use crate::RawFd;

use io_uring_opcode::OpExtEpollCtl;
use io_uring_opcode::{OpCode, OpCompletion, OpError};
use io_uring_owner::Owner;

/// EpollCtl Record
#[derive(Clone, Debug, PartialEq)]
pub struct EpollCtl {
    /// Current owner of the record
    owner: Owner,
    /// Epoll Fd
    epfd: RawFd,
    /// Related filehandle
    fd: RawFd,
    /// Kind of EpollCtl Op
    op: EpollOpKind,
    /// Related Epoll Event
    ev: libc::epoll_event,
}

impl EpollCtl {
    /// Construct a new EpollCtl
    pub fn with_epfd_handled(
        epfd: RawFd,
        handled_fd: HandledFd,
        user_data: u64,
    ) -> Result<Self, EpollCtlError> {
        Ok(EpollCtl {
            owner: Owner::Created,
            epfd,
            fd: handled_fd.fd,
            op: EpollOpKind::Add,
            ev: libc::epoll_event {
                events: handled_fd.wants as u32,
                u64: user_data,
            },
        })
    }
}

/// The kind of EpollCtl Operation
#[derive(Clone, Debug, PartialEq)]
pub enum EpollOpKind {
    /// Add a new entry for Fd
    Add,
    /// Delete the entry of fd
    Delete,
    /// Modify the entry of fd
    Modify,
}

impl EpollOpKind {
    /// The libc equivalent value.
    fn as_libc_i32(&self) -> i32 {
        match self {
            Self::Add => libc::EPOLL_CTL_ADD,
            Self::Delete => libc::EPOLL_CTL_DEL,
            Self::Modify => libc::EPOLL_CTL_MOD,
        }
    }
}

impl OpCompletion for EpollCtl {
    type Error = OpError;
    fn entry(&self) -> io_uring::squeue::Entry {
        let ez_ptr = std::ptr::addr_of!(self.ev);
        io_uring::opcode::EpollCtl::new(
            io_uring::types::Fixed(self.epfd as u32),
            io_uring::types::Fd(self.fd),
            self.op.as_libc_i32(),
            ez_ptr as *const io_uring::types::epoll_event,
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

impl OpCode<EpollCtl> for EpollCtl {
    fn submission(self) -> Result<EpollCtl, OpError> {
        Ok(self)
    }
    fn completion(&mut self, _: Pin<&mut EpollCtl>) -> Result<(), OpError> {
        todo!()
    }
}

impl OpExtEpollCtl for EpollCtl {
    /// Underlying RawFd
    fn raw_fd(&self) -> RawFd {
        self.fd
    }
    /// Underlying libc::eooll_event
    fn ev(&self) -> &libc::epoll_event {
        &self.ev
    }
}
