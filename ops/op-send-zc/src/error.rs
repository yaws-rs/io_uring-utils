//! SendZc op Errors

use core::fmt;
use core::fmt::Display;

/// SendZc Errors
#[derive(Debug)]
pub enum SendZcError {}

impl Display for AcceptMultiError {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl core::error::Error for SendZcError {}
