//! Convenience non-io_uring handler for calls that are not available via io_uring interface.

use crate::error::EpollHandlerError;
use crate::RawFd;

use std::time::Duration;

/// Epoll Handler via syscalls.
#[derive(Debug)]
pub struct EpollHandler {
    epfd: RawFd,
}

impl EpollHandler {
    /// From EpollUringHandler, sharing the epfd
    pub fn from_epoll_uring_handler(h: &crate::EpollUringHandler) -> Self {
        Self::from_epfd(h.epfd())
    }
    /// From epfd RawFd where user is responsible of upholding validity of epfd
    pub fn from_epfd(epfd: RawFd) -> Self {
        Self { epfd }
    }
    /// See epoll_wait(2)
    pub fn wait<const M: usize, F, U>(
        &self,
        t: i32,
        user: &mut U,
        func: F,
    ) -> Result<u32, EpollHandlerError>
    where
        F: Fn(&mut U, u32, u64),
    {
        assert!(M <= u32::MAX as usize);
        let mut evs: [libc::epoll_event; M] = unsafe { std::mem::zeroed() };
        let r = unsafe {
            libc::epoll_wait(
                self.epfd,
                std::ptr::addr_of_mut!(evs) as *mut libc::epoll_event,
                M as i32,
                t,
            )
        };
        if r == -1 {
            // SAFETY: ffi no-data
            let errno = unsafe { libc::__errno_location() };
            return Err(EpollHandlerError::Wait(format!("errno: {:?}", errno)));
        }
        if r == 0 {
            return Ok(0);
        }
        let mut res = 0;
        for i in 1..r {
            let idx: usize = i as usize - 1;
            let events = evs[idx].events;
            let udata_u64 = evs[idx].u64;
            func(user, events, udata_u64);
        }
        Ok(res)
    }
}
