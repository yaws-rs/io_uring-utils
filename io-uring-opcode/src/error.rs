//! General OpCode errors

use core::fmt;
use core::fmt::Display;

/// Harmonized error for OpCode impls creating submission() and completion()
#[derive(Clone, Debug)]
pub enum OpError {
}


impl Display for OpError {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl std::error::Error for OpError {}

