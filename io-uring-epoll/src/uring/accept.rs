//! Accept API Surface

use super::UringHandler;

use super::UringHandlerError; // TODO: COnsider AcceptError?
use crate::fd::{FdKind, RegisteredFd};
use crate::Completion;
use crate::RawFd;

impl UringHandler {
    /// Register Acceptor handle used later with commit_registered_handles                     
    ///                                                                                        
    /// # RawFd Ownership                                                                      
    ///                                                                                        
    /// User is responsibe of owning and ensuring RawFd is valid                               
    pub fn register_acceptor(&mut self, fd: RawFd) -> Result<usize, UringHandlerError> {
        let entry = self.fd_register.vacant_entry();
        let key = entry.key();
        self.fd_register.insert((
            key,
            RegisteredFd {
                kind: FdKind::Acceptor,
                raw_fd: fd,
            },
        ));
        Ok(key)
    }
    /// Add Accept for a IPv4 TCP Listener                                                        
    ///                                                                                           
    /// # Safety                                                                                  
    ///                                                                                           
    /// Use of a `fd` that is not a valid IPv4 TCP Listener is undefined behaviour.               
    pub unsafe fn add_accept_ipv4(&mut self, fd: RawFd) -> Result<(), UringHandlerError> {
        self.add_accept(fd, false)
    }
    /// Add Accept for a IPv6 TCP Listener                                                        
    ///                                                                                           
    /// # Safety                                                                                  
    ///                                                                                           
    /// Use of a `fd` that is not a valid IPv6 TCP Listener is undefined behaviour.               
    pub unsafe fn add_accept_ipv6(&mut self, fd: RawFd) -> Result<(), UringHandlerError> {
        self.add_accept(fd, true)
    }
    pub(crate) unsafe fn add_accept(
        &mut self,
        fd: RawFd,
        v6: bool,
    ) -> Result<(), UringHandlerError> {
        let iou = &mut self.io_uring;
        let mut s_queue = iou.submission();

        let entry = self.fd_slab.vacant_entry();
        let key = entry.key();

        let _k = match v6 {
            true => entry.insert((
                key,
                Completion::Accept(crate::slab::accept::init_accept_rec6()),
            )),
            false => entry.insert((
                key,
                Completion::Accept(crate::slab::accept::init_accept_rec4()),
            )),
        };
        let a_rec_t = self.fd_slab.get(key);
        let dest_slot = None;
        let flags = libc::EFD_NONBLOCK & libc::EFD_CLOEXEC;

        match a_rec_t {
            Some((_k, Completion::Accept(a_rec_k))) => {
                let accept_rec =
                    crate::slab::accept::entry(fd, a_rec_k, dest_slot, flags).user_data(key as u64);
                let _accept = unsafe { s_queue.push(&accept_rec) };
            }
            _ => {
                return Err(UringHandlerError::SlabBugSetGet(
                    "Accept not found after set?",
                ));
            }
        }

        Ok(())
    }
}
