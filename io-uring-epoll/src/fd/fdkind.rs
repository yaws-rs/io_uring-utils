//! Filehandle kinds

use super::HandledFd;

/// Type of Fd mainly used by the safe API
#[derive(Clone, Debug)]
#[allow(dead_code)] // TODO Safe API
pub(crate) enum FdKind {
    /// Epoll ctl handle                  
    EpollCtl,
    /// EPoll Event Handle                
    EpollEvent(HandledFd),
    /// Acceptor handle             
    Acceptor,
    /// Manual handle                      
    Manual,
}
