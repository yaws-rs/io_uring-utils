//! Accept API Surface

use super::UringHandler;
use slabbable::Slabbable;

use super::UringHandlerError; // TODO: COnsider AcceptError?
use crate::Completion;
use crate::RawFd;

impl UringHandler {
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

        let key = match v6 {
            true => self
                .fd_slab
                .take_next_with(Completion::Accept(crate::slab::accept::init_accept_rec6())),
            false => self
                .fd_slab
                .take_next_with(Completion::Accept(crate::slab::accept::init_accept_rec4())),
        }
        .map_err(UringHandlerError::Slabbable)?;
        let a_rec_t = self
            .fd_slab
            .slot_get_ref(key)
            .map_err(UringHandlerError::Slabbable)?;
        let dest_slot = None;
        let flags = libc::EFD_NONBLOCK & libc::EFD_CLOEXEC;

        match a_rec_t {
            Some(Completion::Accept(a_rec_k)) => {
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
