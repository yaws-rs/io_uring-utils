//! FDs registered with Kernel for io_uring

use super::FdKind;
use crate::RawFd;

/// Filehandle registered (or mapped as fixed) with the io_uring API
#[derive(Debug)]
pub(crate) struct RegisteredFd {
    /// Type of registered handle          
    #[allow(dead_code)] // TODO Safe API
    pub(crate) kind: FdKind,
    /// RawFD of the registered handle     
    pub(crate) raw_fd: RawFd,
}
