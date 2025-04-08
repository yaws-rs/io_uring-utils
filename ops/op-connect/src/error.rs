//! Connect op Errors

use core::fmt;
use core::fmt::Display;

use io_uring_bearer::error::UringBearerError;

/// Connect Errors
#[derive(Debug)]
pub enum ConnectError {}

impl Display for ConnectError {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl std::error::Error for ConnectError {}
