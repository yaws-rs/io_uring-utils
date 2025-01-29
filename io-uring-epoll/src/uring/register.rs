//! Register API surface for registering fixed Buffers, filehandles etc.

use super::UringHandler;
use crate::fd::{FdKind, RegisteredFd};
use crate::uring::UringHandlerError;
use crate::RawFd;

impl UringHandler {
    // TODO: keep track of the FdKind and morph it between different types maybe
    //       Acceptor should not become Send/Recv and RecvSend shold become Recv once all Sent .. maybe?
    pub(crate) fn add_registered_fd(
        &mut self,
        reg_fd: RegisteredFd,
    ) -> Result<u32, UringHandlerError> {
        if self.fd_register.len() == self.fd_register.capacity() {
            return Err(UringHandlerError::FdRegisterFull);
        }
        let entry = self.fd_register.vacant_entry();
        let key = entry.key() as u32;
        self.fd_register.insert((key, reg_fd));
        Ok(key)
    }
    /// Register Recv handle
    pub fn register_recv(&mut self, fd: RawFd) -> Result<u32, UringHandlerError> {
        self.add_registered_fd(RegisteredFd::from_raw(fd, FdKind::Recv))
    }
    /// Register Acceptor handle
    pub fn register_acceptor(&mut self, fd: RawFd) -> Result<u32, UringHandlerError> {
        self.add_registered_fd(RegisteredFd::from_raw(fd, FdKind::Acceptor))
    }
    /// Sparse Commit of one rgistered handle
    pub fn commit_registered_sparse(&mut self, key: u32) -> Result<(), UringHandlerError> {
        let raw_fd = match self.fd_register.get(key) {
            Some(&(_, RegisteredFd { raw_fd, .. })) => raw_fd,
            _ => return Err(UringHandlerError::FdNotRegistered(key)),
        };
        match self
            .io_uring()
            .submitter()
            .register_files_update(key, &[raw_fd])
        {
            Ok(_) => Ok(()),
            // TODO: This looks like it's infallible but..?
            Err(_) => Err(UringHandlerError::FdRegisterFail),
        }
    }
    /// Commit the full table of currently registered and free capacity into the kernel.
    ///
    /// # Warning
    ///
    /// Generally make sure the queues are / can go empty before calling this.
    pub fn commit_registered_init(&mut self) -> Result<(), UringHandlerError> {
        let iou = &mut self.io_uring;
        let submitter = iou.submitter();

        let mut vec_i32: Vec<RawFd> = Vec::with_capacity(self.fd_register.capacity() as usize);
        for idx in 0..self.fd_register.capacity() {
            if let Some((_, itm)) = self.fd_register.get(idx) {
                vec_i32.push(itm.raw_fd);
            } else {
                vec_i32.push(-1);
            }
        }

        let slice_i32: &[i32] = &vec_i32[0..];

        submitter
            .register_files(slice_i32)
            .map_err(|e| UringHandlerError::RegisterHandles(e.to_string()))?;

        Ok(())
    }
}
