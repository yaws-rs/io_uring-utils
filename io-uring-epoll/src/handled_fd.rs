//! HandledFd

use std::os::fd::RawFd;

/// HandledFd Instance
#[derive(Clone, Debug, PartialEq)]
pub struct HandledFd {
    pub(crate) fd: RawFd,
    pub(crate) wants: i32,
    pub(crate) pending: Option<i32>,
    pub(crate) committed: Option<i32>,
    pub(crate) error: Option<String>,
    pub(crate) current_submission: Option<usize>,
}

impl HandledFd {
    /// Create a new EpollHandler associated [`HandledFd`]                        
    pub fn from_raw(fd: RawFd) -> Self {
        HandledFd {
            fd,
            wants: 0,
            committed: None,
            current_submission: None,
            error: None,
            pending: None,
        }
    }
    /// Extract RawFd                                                                 
    pub fn as_raw(&self) -> RawFd {
        self.fd
    }
    // All setters
    fn turn_on_or_off(&mut self, mask_in: i32, on_or_off: bool) -> i32 {
        let cur_wants: i32 = self.wants;
        self.wants = match on_or_off {
            true => cur_wants | mask_in,
            false => cur_wants ^ mask_in,
        };
        self.wants
    }
    /// Set EPOLLIN per epoll.h in userspace On or Off                                            
    /// Returns returns raw mask as to be sent to kernel                           
    /// Use [`EpollHandler::prepare_submit()`] after                              
    pub fn set_in(&mut self, on_or_off: bool) -> i32 {
        self.turn_on_or_off(libc::EPOLLIN, on_or_off)
    }
    /// EPOLLPRI                                                          
    pub fn set_pri(&mut self, on_or_off: bool) -> i32 {
        self.turn_on_or_off(libc::EPOLLPRI, on_or_off)
    }
    /// EPOLLOUT                                                                    
    pub fn set_out(&mut self, on_or_off: bool) -> i32 {
        self.turn_on_or_off(libc::EPOLLOUT, on_or_off)
    }
    /// EPOLLERR                                                                  
    pub fn set_err(&mut self, on_or_off: bool) -> i32 {
        self.turn_on_or_off(libc::EPOLLERR, on_or_off)
    }
    /// EPOLLHUP                                                                  
    pub fn set_hup(&mut self, on_or_off: bool) -> i32 {
        self.turn_on_or_off(libc::EPOLLHUP, on_or_off)
    }
    /// EPOLLRDNORM                                                                   
    pub fn set_rdnorm(&mut self, on_or_off: bool) -> i32 {
        self.turn_on_or_off(libc::EPOLLRDNORM, on_or_off)
    }
    /// EPOLLRDBAND                                                                          
    pub fn set_rdband(&mut self, on_or_off: bool) -> i32 {
        self.turn_on_or_off(libc::EPOLLRDBAND, on_or_off)
    }
    /// EPOLLWRNORM                                                     
    pub fn set_wrnorm(&mut self, on_or_off: bool) -> i32 {
        self.turn_on_or_off(libc::EPOLLWRNORM, on_or_off)
    }
    /// EPOLLWRBAND per epoll.h userspace On or Off                     
    pub fn set_wrband(&mut self, on_or_off: bool) -> i32 {
        self.turn_on_or_off(libc::EPOLLWRBAND, on_or_off)
    }
    /// EPOLLMSG                                                                  
    pub fn set_msg(&mut self, on_or_off: bool) -> i32 {
        self.turn_on_or_off(libc::EPOLLMSG, on_or_off)
    }
    /// EPOLLRDHUP                                                                
    pub fn set_rdhup(&mut self, on_or_off: bool) -> i32 {
        self.turn_on_or_off(libc::EPOLLRDHUP, on_or_off)
    }
    /// EPOLLWAKEUP                                                                   
    pub fn set_wakeup(&mut self, on_or_off: bool) -> i32 {
        self.turn_on_or_off(libc::EPOLLWAKEUP, on_or_off)
    }
    /// EPOLLONESHOT                                                                         
    pub fn set_oneshot(&mut self, on_or_off: bool) -> i32 {
        self.turn_on_or_off(libc::EPOLLONESHOT, on_or_off)
    }
    /// EPOLLET                                                         
    pub fn set_et(&mut self, on_or_off: bool) -> i32 {
        self.turn_on_or_off(libc::EPOLLET, on_or_off)
    }
    /// Get the raw u32 Epoll event mask as set in userspace                      
    /// This may not have been sent and may be pending send or not committed      
    /// Use [`EpollHandler::prepare_submit()`] after                              
    pub fn get_mask_raw(&mut self) -> i32 {
        self.wants
    }
    /// Set the raw u32 Epoll event mask in the userspace                         
    /// *WARNING*: Ensure this is valid per epoll.h of your kernel                              
    /// Use [`EpollHandler::prepare_submit()`] after                                  
    pub fn set_mask_raw(&mut self, mask: i32) {
        self.wants = mask;
    }
    /// Get the pending eq u32 Epoll                                                         
    /// This may not be committed into kernel yet use get_committed to check                    
    /// This will be none if there is no pending change or it has not been sent       
    /// Use [`EpollHandler::prepare_submit()`] after                           
    pub fn get_pending(&self) -> Option<i32> {
        self.pending
    }
}

#[cfg(test)]
mod test {
    use super::HandledFd;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
    use std::os::fd::AsRawFd;

    fn handle_fd() -> HandledFd {
        let s =
            TcpListener::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0)).unwrap();
        HandledFd::from_raw(s.as_raw_fd())
    }

    #[test]
    fn mask_fd_inouts() {
        let mut fd = handle_fd();
        fd.set_in(true);
        assert_eq!(fd.set_out(true), 5);
        assert_eq!(fd.set_in(false), 4);
        assert_eq!(fd.set_out(false), 0);
    }
    #[test]
    fn mask_fd_in() {
        let mut fd = handle_fd();
        assert_eq!(fd.set_in(true), 1);
        assert_eq!(fd.set_in(false), 0);
    }
    #[test]
    fn mask_fd_pri() {
        let mut fd = handle_fd();
        assert_eq!(fd.set_pri(true), 2);
        assert_eq!(fd.set_pri(false), 0);
    }
    #[test]
    fn mask_fd_out() {
        let mut fd = handle_fd();
        assert_eq!(fd.set_out(true), 4);
        assert_eq!(fd.set_out(false), 0);
    }
    #[test]
    fn mask_fd_err() {
        let mut fd = handle_fd();
        assert_eq!(fd.set_err(true), 8);
        assert_eq!(fd.set_err(false), 0);
    }
    #[test]
    fn mask_fd_hup() {
        let mut fd = handle_fd();
        assert_eq!(fd.set_hup(true), 0x00000010);
        assert_eq!(fd.set_hup(false), 0);
    }
    #[test]
    fn mask_fd_rdnorm() {
        let mut fd = handle_fd();
        assert_eq!(fd.set_rdnorm(true), 0x00000040);
        assert_eq!(fd.set_rdnorm(false), 0);
    }
    #[test]
    fn mask_fd_rdband() {
        let mut fd = handle_fd();
        assert_eq!(fd.set_rdband(true), 0x00000080);
        assert_eq!(fd.set_rdband(false), 0);
    }
    #[test]
    fn mask_fd_wrnorm() {
        let mut fd = handle_fd();
        assert_eq!(fd.set_wrnorm(true), 0x00000100);
        assert_eq!(fd.set_wrnorm(false), 0);
    }
    #[test]
    fn mask_fd_wrband() {
        let mut fd = handle_fd();
        assert_eq!(fd.set_wrband(true), 0x00000200);
        assert_eq!(fd.set_wrband(false), 0);
    }
    #[test]
    fn mask_fd_msg() {
        let mut fd = handle_fd();
        assert_eq!(fd.set_msg(true), 0x00000400);
        assert_eq!(fd.set_msg(false), 0);
    }
    #[test]
    fn mask_fd_rdhup() {
        let mut fd = handle_fd();
        assert_eq!(fd.set_rdhup(true), 0x00002000);
        assert_eq!(fd.set_rdhup(false), 0);
    }
    #[test]
    fn mask_fd_wakeup() {
        let mut fd = handle_fd();
        assert_eq!(fd.set_wakeup(true), 0x20000000);
        assert_eq!(fd.set_wakeup(false), 0);
    }
    #[test]
    fn mask_fd_oneshot() {
        let mut fd = handle_fd();
        assert_eq!(fd.set_oneshot(true), 0x40000000);
        assert_eq!(fd.set_oneshot(false), 0);
    }
    #[test]
    fn mask_fd_et() {
        let mut fd = handle_fd();
        assert_eq!(fd.set_et(true) as u32, 0x80000000);
        assert_eq!(fd.set_et(false), 0);
    }
}
