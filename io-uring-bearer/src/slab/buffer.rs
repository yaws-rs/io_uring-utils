//! ProvideBuffers Slab Record

use io_uring_owner::{Owner, TakeError};

use std::marker::PhantomPinned;
use std::pin::Pin;

/// Holds the actual allocation for the Buffers that either owned by the Kernel or Userspace.
#[derive(Clone, Debug)]
pub struct BuffersRec {
    pub(crate) owner: Owner,
    all_bufs: Vec<u8>,
    len_per_buf: i32,
    num_bufs: u16,

    _pin: PhantomPinned,
}

impl BuffersRec {
    /// Take the buffer for filling it.
    pub(crate) unsafe fn take_for_filling(&mut self) -> Pin<&mut Vec<u8>> {
        std::pin::Pin::new(&mut self.all_bufs)
    }
    /// Typically you do not use this directly but instead via provide_buffers that uses this.
    pub(crate) fn force_owner_kernel(&mut self) {
        self.owner = Owner::Kernel;
    }
    /// Get the current ownership status.
    pub fn owner(&self) -> Owner {
        self.owner.clone()
    }
    /// Lenght per buffer in BuffersRec
    pub fn len_per_buf(&self) -> i32 {
        self.len_per_buf
    }
    /// Number of buffers in BuffersRec
    pub fn num_bufs(&self) -> u16 {
        self.num_bufs
    }
    /// All buffers.
    ///
    /// # Safety
    ///
    /// The whole set of buffers may have been provided to kernel as mutable.
    pub unsafe fn all_bufs(&self) -> &Vec<u8> {
        &self.all_bufs
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
    slab_idx: Option<usize>,
    buf: *mut u8,
    len_per_buf: i32,
    num_bufs: u16,
    bgid: u16,
    bid: u16,
}

impl ProvideBuffersRec {
    /// SLab index
    pub fn slab_idx(&self) -> Option<usize> {
        self.slab_idx
    }
}

/// Mutable Buffer is taken by something, let's provide it intermediate type.  
#[derive(Clone, Debug)]
pub(crate) struct TakenMutableBuffer {
    pub(crate) buf_idx: usize,
    pub(crate) buf_mut_u8: *mut u8,
    pub(crate) buf_size: u32,
}

#[inline]
pub(crate) fn take_one_mutable_buffer_raw(
    buf_idx: usize,
    buf_rec: &mut BuffersRec,
) -> Result<TakenMutableBuffer, TakeError> {
    if buf_rec.num_bufs() != 1 {
        return Err(TakeError::OnlyOneTakeable);
    }
    buf_rec.owner.take()?;
    Ok(TakenMutableBuffer {
        buf_idx,
        buf_size: buf_rec.len_per_buf as u32,
        buf_mut_u8: &raw mut buf_rec.all_bufs as *mut u8,
    })
}

/// Buffer is taken by something for Read-Only, let's provide it intermediate type.  
#[derive(Clone, Debug)]
pub(crate) struct TakenImmutableBuffer {
    pub(crate) buf_idx: usize,
    pub(crate) buf_const_u8: *const u8,
    pub(crate) buf_size: u32,
    pub(crate) buf_kernel_index: u16,
}

#[inline]
pub(crate) fn take_one_immutable_buffer_raw(
    buf_idx: usize,
    buf_kernel_index: u16,
    buf_rec: &mut BuffersRec,
) -> Result<TakenImmutableBuffer, TakeError> {
    if buf_rec.num_bufs() != 1 {
        return Err(TakeError::OnlyOneTakeable);
    }
    buf_rec.owner.take()?;
    Ok(TakenImmutableBuffer {
        buf_idx,
        buf_size: buf_rec.len_per_buf as u32,
        buf_const_u8: buf_rec.all_bufs.as_ptr(),
        buf_kernel_index,
    })
}

#[inline]
pub(crate) fn provide_buffer_rec(
    bgid: u16,
    bid: u16,
    buf: &mut BuffersRec,
    slab_idx: Option<usize>,
) -> ProvideBuffersRec {
    ProvideBuffersRec {
        slab_idx,
        buf: buf.all_bufs.as_mut_ptr(),
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
