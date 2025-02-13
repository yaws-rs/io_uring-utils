//! EpollCtl Record

use crate::fd::HandledFd;
use crate::Owner;
use crate::RawFd;

/// EpollCtl Record
#[derive(Clone, Debug, PartialEq)]
pub struct EpollRec {
    /// Related filehandle
    fd: RawFd,
    /// Current owner of the record
    owner: Owner,
    /// Related Epoll Event
    ev: libc::epoll_event,
}

impl EpollRec {
    /// Underlying RawFd
    pub fn fd(&self) -> RawFd {
        self.fd
    }
    /// Underlying libc::eooll_event
    pub fn ev(&self) -> libc::epoll_event {
        self.ev
    }
    /// Current ownership
    pub fn owner(&self) -> Owner {
        self.owner.clone()
    }
}

#[inline]
pub(crate) fn event_rec(handled_fd: &HandledFd, user_data: u64) -> EpollRec {
    EpollRec {
        fd: handled_fd.fd,
        owner: Owner::Created,
        ev: libc::epoll_event {
            events: handled_fd.wants as u32,
            u64: user_data,
        },
    }
}

#[inline]
pub(crate) fn add(epfd: RawFd, event_rec: &EpollRec) -> io_uring::squeue::Entry {
    entry(epfd, libc::EPOLL_CTL_ADD, event_rec)
}

#[inline]
fn entry(epfd: RawFd, epoll_op: i32, event_rec: &EpollRec) -> io_uring::squeue::Entry {
    let ez_ptr = std::ptr::addr_of!(event_rec.ev);
    io_uring::opcode::EpollCtl::new(
        io_uring::types::Fixed(epfd as u32),
        io_uring::types::Fd(event_rec.fd),
        epoll_op,
        ez_ptr as *const io_uring::types::epoll_event,
    )
    .build()
}
