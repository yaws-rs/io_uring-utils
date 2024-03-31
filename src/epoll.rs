//! Higher level EpollHandler abstraction to deal with Linux epoll via io_uring

pub enum EpollHandlerError {
}

pub struct EpollHandler;

impl EpollHandler {
    pub fn new(
    /// Create new handler from existing io-uring::IoUring builder
    pub fn from_io_uring(iou: IoUring<io_uring::squeue::Entry, io_uring::cqueue::Entry>) -> Result<(), EpollHandlerError> {
    }
}
