//! Filehandle types

mod fdkind;
mod handled_fd;
mod registered_fd;

pub use handled_fd::HandledFd;

pub(crate) use fdkind::*;
pub(crate) use registered_fd::*;
