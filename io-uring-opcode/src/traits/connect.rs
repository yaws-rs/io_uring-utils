//! Connect extension trait

use ysockaddr::YSockAddrC;

/// Connect Expansion trait
pub trait OpExtConnect {
    /// Underlying Fixed Fd ref
    fn fixed_fd(&self) -> u32;
    /// Underlying YSockAddr
    fn ysaddr(&self) -> &YSockAddrC;
}
