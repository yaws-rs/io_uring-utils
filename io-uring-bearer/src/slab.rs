//! Slab Types
//! These types are typically stored in slab / slotmap that can gurantee
//! memory address would not change for example when used in ffi calls
//! which may complete asynchronously and may where the records may be
//! retained beyond the initial call.

// Accept async return stores IP + Port of accepted (new) socket
pub(crate) mod accept;
#[doc(inline)]
pub use accept::AcceptRec;

// Buffers are shared between the kernel and userspace and may be returned
// on Read/Recv etc. calls
pub(crate) mod buffer;
#[doc(inline)]
pub use buffer::{BuffersRec, ProvideBuffersRec};

pub(crate) mod futex;
#[doc(inline)]
pub use futex::{FutexRec, FutexWaitRec};

pub(crate) mod recv;
#[doc(inline)]
pub use recv::{RecvMultiRec, RecvRec};

pub(crate) mod send_zc;
#[doc(inline)]
pub use send_zc::SendZcRec;
