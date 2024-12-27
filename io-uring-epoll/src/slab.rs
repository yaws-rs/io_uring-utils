//! Slab Types
//! These types are typically stored in slab / slotmap that can gurantee
//! memory address would not change for example when used in ffi calls
//! which may complete asynchronously and may where the records may be
//! retained beyond the initial call.

pub(crate) mod accept;
#[doc(inline)]
pub use accept::AcceptRec;

pub(crate) mod epoll_ctl;
#[doc(inline)]
pub use epoll_ctl::EpollRec;
