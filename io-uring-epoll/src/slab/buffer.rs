//! ProvideBuffers Slab Record

use crate::Owner;
use std::marker::PhantomPinned;

/// Holds the actual allocation for the Buffers that either owned by the Kernel or Userspace.
#[derive(Clone, Debug)]
pub struct BuffersRec {
    owner: Owner,
    all_bufs: Vec<u8>,
    len_per_buf: i32,
    num_bufs: u16,

    _pin: PhantomPinned,
}

impl BuffersRec {
    /// Typically you do not use this directly but instead via provide_buffers that uses this.
    pub(crate) fn force_owner_kernel(&mut self) {
        self.owner = Owner::Kernel;
    }
    /// Get the current ownership status.
    pub fn owner(&self) -> Owner {
        self.owner.clone()
    }
}

#[inline]
pub(crate) fn construct_buffer(owner: Owner, len_per_buf: i32, num_bufs: u16) -> BuffersRec {
    assert!(len_per_buf > 0);
    assert!(num_bufs > 0);
    // TODO: overflow maybe - stupid casts
    let total_siz: usize = len_per_buf as usize * num_bufs as usize;
    let v: Vec<u8> = vec![0; total_siz];
    BuffersRec {
        owner,
        len_per_buf,
        num_bufs,
        all_bufs: v,
        _pin: PhantomPinned,
    }
}

/// The stored Submission & Completion record for ProvideBuffers.
/// This can be dropped immediately after completion as BufferRec is separated.
#[derive(Clone, Debug)]
pub struct ProvideBuffersRec {
    buf: *mut u8,
    len_per_buf: i32,
    num_bufs: u16,
    bgid: u16,
    bid: u16,
}

#[inline]
pub(crate) fn provide_buffer_rec(bgid: u16, bid: u16, buf: &mut BuffersRec) -> ProvideBuffersRec {
    ProvideBuffersRec {
        buf: &raw mut buf.all_bufs as *mut u8,
        len_per_buf: buf.len_per_buf,
        num_bufs: buf.num_bufs,
        bgid,
        bid,
    }
}

#[inline]
pub(crate) fn entry(rec: &ProvideBuffersRec) -> io_uring::squeue::Entry {
    //    let mut buf_in: [u8; 16384] = unsafe { std::mem::zeroed() };
    io_uring::opcode::ProvideBuffers::new(
        //        std::ptr::addr_of_mut!(rec.buf.all_bufs) as *mut u8,
        rec.buf,
        rec.len_per_buf, // len
        rec.num_bufs,    // nbufs
        rec.bgid,        // bgid
        rec.bid,         // bid
    )
    .build()
}
