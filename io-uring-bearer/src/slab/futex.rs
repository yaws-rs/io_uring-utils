//! Futex Slab Records

use crate::Owner;
use std::sync::atomic::AtomicU32;
use std::sync::Arc;

/// Holds the actual Futexes or References (via unsafe)
#[derive(Clone, Debug)]
pub enum FutexRec {
    /// Owned Futex Atomic
    Owned(FutexOwnedRec),
    /// Unsafe referred Futex Atomic
    UnsafeReferenced(FutexUnsafeRec),
}

impl FutexRec {
    /// Where the Futex atomic is at?
    pub fn owner(&self) -> Owner {
        match self {
            FutexRec::Owned(r) => r.pending_at.clone(),
            FutexRec::UnsafeReferenced(r) => r.pending_at.clone(),
        }
    }
    /// You should not use this directly but available for custom impleentations.
    pub fn force_owner_kernel(&mut self) {
        match self {
            FutexRec::Owned(r) => r.pending_at = Owner::Kernel,
            FutexRec::UnsafeReferenced(r) => r.pending_at = Owner::Kernel,
        }
    }
}

// TODO: Maybe provide intermediate-referenced type which could be naively tiny bit safer.

/// Owned Futex Atomic
#[derive(Clone, Debug)]
pub struct FutexOwnedRec {
    owned_atom: Arc<AtomicU32>,
    // Must not move / invalidate / remove etc. if Kernel refers to it.
    pending_at: Owner,
}

impl FutexOwnedRec {
    pub(crate) fn atom_arc(&self) -> Arc<AtomicU32> {
        self.owned_atom.clone()
    }
}

/// Referenced (Unsafe) Futex Atomic
#[derive(Clone, Debug)]
pub struct FutexUnsafeRec {
    unsafe_atom: *const u32,
    // Must not move / invalidate / remove etc. if Kernel refers to it.
    pending_at: Owner,
}

#[inline]
pub(crate) fn construct_futex() -> FutexRec {
    FutexRec::Owned(FutexOwnedRec {
        owned_atom: Arc::new(AtomicU32::new(0)),
        pending_at: Owner::default(),
    })
}

#[inline]
pub(crate) fn construct_unsafe_futex(f: *const u32) -> FutexRec {
    FutexRec::UnsafeReferenced(FutexUnsafeRec {
        unsafe_atom: f,
        pending_at: Owner::default(),
    })
}

/// FutexWait Completion Rec
#[derive(Clone, Debug, PartialEq)]
pub struct FutexWaitRec {
    atom: *const u32,
    bitset: u64,
    val: u64,
}

#[inline]
pub(crate) fn wait_futex_rec(bitset: u64, val: u64, futex_rec: &FutexRec) -> FutexWaitRec {
    let atom = match futex_rec {
        FutexRec::Owned(owned_rec) => owned_rec.owned_atom.as_ptr() as *const u32,
        FutexRec::UnsafeReferenced(unsafe_rec) => unsafe_rec.unsafe_atom,
    };
    FutexWaitRec { atom, bitset, val }
}

// From: https://github.com/torvalds/linux/blob/v6.7/include/uapi/linux/futex.h#L63
const FUTEX2_SIZE_U32: u32 = 2;

#[inline]
pub(crate) fn entry(rec: &FutexWaitRec) -> io_uring::squeue::Entry {
    io_uring::opcode::FutexWait::new(rec.atom, rec.val, rec.bitset, FUTEX2_SIZE_U32).build()
}
