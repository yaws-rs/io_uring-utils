//! EpollCtl extension trait

/// EpollCtl Expansion trait
pub trait OpExtEpollCtl {
    /// Underlying RawFd
    fn raw_fd(&self) -> std::os::fd::RawFd;
    /// Underlying libc::eooll_event
    fn ev(&self) -> &libc::epoll_event;
}
