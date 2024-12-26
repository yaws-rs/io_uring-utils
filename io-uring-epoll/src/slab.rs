//! Slab Types
//! These types are typically stored in slab / slotmap that can gurantee
//! memory address would not change for example when used in ffi calls
//! which may complete asynchronously and may where the records may be
//! retained beyond the initial call.

mod accept;
mod epoll_ctl;
