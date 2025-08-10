//! Socket op Errors

use core::fmt;
use core::fmt::Display;

/// Socket Errors
#[derive(Debug)]
pub enum SocketError {
    /// Must target valid fixed fd, Usually between 0 .. u32_MAX-2
    InvalidTarget(u32),
}

impl Display for SocketError {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl std::error::Error for SocketError {}
