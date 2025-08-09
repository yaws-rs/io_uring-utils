//! Connect extension trait

use crate::types::TargetFdType;

/// Connect Expansion trait
pub trait OpExtSocket {
    /// Domain, e.g. AF_INET
    fn domain(&self) -> i32;
    /// Socket type, e.g. SOCK_STREAM
    fn socket_type(&self) -> i32;
    /// Protocol, e.g. IPPROTO_TCP
    fn protocol(&self) -> i32;
    /// Targeted Fd Type and slot if any
    fn target_fd(&self) -> TargetFdType;
}
