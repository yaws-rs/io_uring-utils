//! Slab Types
//! These types are typically stored in slab / slotmap that can gurantee
//! memory address would not change for example when used in ffi calls
//! which may complete asynchronously and may where the records may be
//! retained beyond the initial call.

// Accept async return stores IP + Port of accepted (new) socket
pub(crate) mod accept;
#[doc(inline)]
pub use accept::AcceptRec;

// Epoll on wait returns user_data u64 and triggered events (we init-zeroo)
pub(crate) mod epoll_ctl;
#[doc(inline)]
pub use epoll_ctl::EpollRec;

// Buffers are shared between the kernel and userspace and may be returned
// on Read/Recv etc. calls
pub(crate) mod buffer;
#[doc(inline)]
pub use buffer::{BuffersRec, ProvideBuffersRec};
