//! Connect extension trait

/// Connect Expansion trait
pub trait OpExtConnect {
    /// Underlying RawFd
    fn raw_fd(&self) -> std::os::fd::RawFd;
//    /// Underlying libc::eooll_event
//    fn ev(&self) -> &libc::epoll_event;
}
