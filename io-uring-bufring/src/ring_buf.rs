//! Uring RingBuf

use crate::RingBufError;
use core::num::NonZero;
use core::sync::atomic::{AtomicU16, Ordering};

use io_uring::types::BufRingEntry;

use anonymous_mmap::AnonymousMmap;

/// The total desired amount of buffers in the ringbuf
pub struct BufferCount(pub NonZero<u16>);

/// The desired size of each buffer
pub struct PerBufferSize(pub NonZero<u16>);

/// The Pagesize used
pub struct PageSize(pub NonZero<u16>);

/// Pick the desired ringbuf capacity. Note that the Linux default
/// pagesize is 4kB which we mostly assume will be used here.
///
/// To use custom pagesize, use with the appropriate values that aligns.
#[derive(Debug)]
pub struct RingBufChoice {
    bufs_count: u16,
    per_buf_size: u16,
    total_bufs_size: usize,
}

impl RingBufChoice {
    /// Construct the choice checked against the Linux default pagesize which is 4 kB
    #[inline]
    pub fn with_default_pagesize(
        bufs_count: BufferCount,
        per_buf_size: PerBufferSize,
    ) -> Result<Self, RingBufError> {
        // SAFETY: The value supplied is non-zero literal
        Self::with_custom_pagesize(
            bufs_count,
            per_buf_size,
            PageSize(unsafe { NonZero::new_unchecked(4096) }),
        )
    }
    /// Construct choice checked against custom page size
    #[inline]
    pub fn with_custom_pagesize(
        bufs_count: BufferCount,
        per_buf_size: PerBufferSize,
        page_size: PageSize,
    ) -> Result<Self, RingBufError> {
        let bufs_count = bufs_count.0.get();
        let per_buf_size = per_buf_size.0.get();
        let page_size = page_size.0.get();

        if per_buf_size % page_size != 0 {
            return Err(RingBufError::PageSizeUndivisible);
        }

        let total_bufs_size = bufs_count as usize * per_buf_size as usize;

        Ok(Self {
            bufs_count,
            per_buf_size,
            total_bufs_size,
        })
    }
    /// Construct a choice using a page size that is not checked against the given per buffer size
    #[inline]
    pub fn with_unchecked(bufs_count: BufferCount, per_buf_size: PerBufferSize) -> Self {
        let bufs_count = bufs_count.0.get();
        let per_buf_size = per_buf_size.0.get();
        let total_bufs_size = bufs_count as usize * per_buf_size as usize;
        Self {
            bufs_count,
            per_buf_size,
            total_bufs_size,
        }
    }
}

impl RingBufChoice {
    /// The number of buffer entries in the ring
    #[inline]
    pub const fn total_bufs_count(&self) -> u16 {
        self.bufs_count
    }
    /// The total size of all buffers combined
    #[inline]
    pub const fn total_bufs_size(&self) -> usize {
        self.total_bufs_size
    }
    /// Buffer size per buffer
    #[inline]
    pub const fn per_bufsize(&self) -> u16 {
        self.per_buf_size
    }
}

/// Unregistered ring buffer for the purposes of using it within io_uring.
/// Ring buffer mapping avoids registering buffers one by one.
/// After constructing the ringbuf you can register it either with the bearer
/// using the [`RingBufUnregistered::register_with_bearer`] or your using
/// your own io_uring handler outside this crate.
#[derive(Debug)]
pub struct RingBufUnregistered {
    choice: RingBufChoice,
    ring_start: AnonymousMmap,
}

impl RingBufUnregistered {
    /// Create unregistered io_uring ringbuf with the given ringbuf choice and raw buffer.
    ///
    /// # Safety
    ///
    /// The caller must ensure the raw buffer is valid for the entire mapped range starting
    /// from the pointer continuously within the bounds of the given choice.
    ///
    /// When this ring is used within the io_uring, the raw buffer must stay valid and ummoved
    /// for the whole time it's registered.
    #[inline]
    pub unsafe fn with_rawbuf_continuous(
        choice: RingBufChoice,
        base_ptr: *mut u8,
    ) -> Result<Self, RingBufError> {
        let entry_size = size_of::<BufRingEntry>(); // 16B
        let ring_size = entry_size * choice.total_bufs_count() as usize;
        let ring_start = AnonymousMmap::new(ring_size).map_err(RingBufError::Mmap)?;

        for bid in 0u16..choice.total_bufs_count() {
            let entries = ring_start.as_ptr_mut() as *mut BufRingEntry;
            // SAFETY: Our owned AnonymousMmap holds the validity
            let new_entry = unsafe { &mut *entries.add(bid as usize) };
            // SAFETY: Our owned AnonymousMmap holds the validity
            let aligned_bid_ptr = unsafe { base_ptr.add(bid as usize * 8192) };
            new_entry.set_addr(aligned_bid_ptr as _);
            new_entry.set_len(8192 as _);
            new_entry.set_bid(bid);
        }

        // SAFETY: Our owned AnonymousMmap holds the validity
        let shared_tail = unsafe { BufRingEntry::tail(ring_start.as_ptr() as *const BufRingEntry) }
            as *const AtomicU16;

        // SAFETY: Nobody else is modifying it.
        unsafe {
            (*shared_tail).store(256, Ordering::Release);
        }

        Ok(Self { choice, ring_start })
    }
    /// Provides the ring start mut ptr e.g. using it to register it with io_uring.
    ///
    /// ## Safety
    ///
    /// The construt behind is intended for internal usage and may be invalid if the
    /// data behind the pointer is invalidated.
    #[inline]
    pub unsafe fn as_mut_ptr(&mut self) -> *mut libc::c_void {
        self.ring_start.as_ptr_mut()
    }
    /// Provides the count of total number of buffers in the ring.
    #[inline]
    pub fn total_bufs_count(&self) -> u16 {
        self.choice.total_bufs_count()
    }
    /// Register the unregistered ring buffer with the given bearer and buffer group id.
    #[inline]
    #[cfg(feature = "bearer")]
    pub fn register_with_bearer<W>(
        self,
        bearer: &mut io_uring_bearer::UringBearer<W>,
        bgid: u16,
    ) -> Result<RingBufRegistered, RingBufError>
    where
        W: core::fmt::Debug + Clone + io_uring_opcode::OpCompletion,
    {
        // SAFETY: We hold the underlying ringbuf and keep it valid
        let r = unsafe {
            bearer.io_uring().submitter().register_buf_ring_with_flags(
                self.ring_start.as_ptr_mut() as _,
                self.total_bufs_count(), // TODO: hardcoded atm. abstract this in hugetbl
                bgid,
                io_uring::types::IOU_PBUF_RING_INC as u16,
            )
            // TODO: errno mapping probables EINVAL - Needs kernel 5.19+, EEXIST - DUP bgid
        };

        match r {
            Err(e) => Err(RingBufError::Register(self, e)),
            Ok(()) => Ok(RingBufRegistered {
                inner_ring: self,
                bgid,
            }),
        }
    }
}

/// Registered RingBuf
#[cfg(feature = "bearer")]
#[derive(Debug)]
pub struct RingBufRegistered {
    inner_ring: RingBufUnregistered,
    bgid: u16,
}

#[cfg(feature = "bearer")]
impl RingBufRegistered {
    /// Register the unregistered ring buffer with the given bearer and buffer group id.
    #[inline]
    #[cfg(feature = "bearer")]
    pub fn unregister_with_bearer<W>(
        self,
        bearer: &mut io_uring_bearer::UringBearer<W>,
    ) -> Result<RingBufUnregistered, RingBufError>
    where
        W: core::fmt::Debug + Clone + io_uring_opcode::OpCompletion,
    {
        let r = bearer.io_uring().submitter().unregister_buf_ring(self.bgid);

        match r {
            Err(e) => Err(RingBufError::Unregister(self, e)),
            Ok(()) => Ok(self.inner_ring),
        }
    }
}

#[cfg(test)]
mod ring_buf_test;
