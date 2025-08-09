//! AcceptMulti op Errors

use core::fmt;
use core::fmt::Display;

/// AcceptMulti Errors
#[derive(Debug)]
pub enum AcceptMultiError {}

impl Display for AcceptMultiError {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl std::error::Error for AcceptMultiError {}
