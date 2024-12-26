//! EpollCtl Builder

use crate::HandledFd;
use crate::RawFd;

#[derive(Clone, Debug, PartialEq)]
pub struct EpollRec {
    fd: RawFd,
    ev: libc::epoll_event,
}

#[inline]
pub(super) fn event_rec(handled_fd: &HandledFd, user_data: u64) -> EpollRec {
    EpollRec {
        fd: handled_fd.fd,
        ev: libc::epoll_event {
            events: handled_fd.wants as u32,
            u64: user_data,
        },
    }
}

#[inline]
pub(super) fn add(epfd: RawFd, event_rec: &EpollRec) -> io_uring::squeue::Entry {
    entry(epfd, libc::EPOLL_CTL_ADD, event_rec)
}

#[inline]
fn entry(epfd: RawFd, epoll_op: i32, event_rec: &EpollRec) -> io_uring::squeue::Entry {
    let ez_ptr = std::ptr::addr_of!(event_rec.ev);
    let uring_submission_rec = io_uring::opcode::EpollCtl::new(
        io_uring::types::Fixed(epfd as u32),
        io_uring::types::Fd(event_rec.fd),
        epoll_op,
        ez_ptr as *const io_uring::types::epoll_event,
    )
    .build();

    uring_submission_rec
}
