//! Associated related types

#[cfg(feature = "socket")]
pub(crate) mod socket;
#[cfg(feature = "socket")]
pub use socket::TargetFdType;
