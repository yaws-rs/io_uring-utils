//! SendZc Slab records

use io_uring_owner::Owner;

use crate::slab::buffer::TakenImmutableBuffer;

/// SendZc could be either through a fixed registered buffer through it's index or
/// unsafely provided through as raw reference.
#[derive(Clone, Debug)]
pub enum SendZcRec {
    /// Fixed registered buffer
    Fixed(SendZcFixedRec),
    /// Unsafely provided through raw reference
    UnsafeRef(SendZcUnsafeRefRec),
}

/// SendZc Fixed Record
#[derive(Clone, Debug)]
pub struct SendZcFixedRec {
    fixed_fd: u32,
    owner: Owner,
    buf_taken: TakenImmutableBuffer,
    #[allow(dead_code)]
    to_addr: Option<DestTo>,
}

/// SendZc UnsafeRef Record
#[derive(Clone, Debug)]
pub struct SendZcUnsafeRefRec {
    fixed_fd: u32,
    owner: Owner,
    buf_const_u8: *const u8,
    buf_size: u32,
    #[allow(dead_code)]
    to_addr: Option<DestTo>,
}

impl SendZcRec {
    /// Zero-Copy Send with a previously Bearer created by and managed buffer.
    #[inline]
    pub(crate) fn with_fixed_buf(fixed_fd: u32, buf_taken: TakenImmutableBuffer) -> Self {
        SendZcRec::Fixed(SendZcFixedRec {
            fixed_fd,
            owner: Owner::Created,
            buf_taken,
            to_addr: None, // TODO
        })
    }
    /// Zero-Copy Send with the supplied raw buffer which not managed by the bearer.
    #[inline]
    pub(crate) unsafe fn with_unsafe_rawbuf(
        fixed_fd: u32,
        buf_const_u8: *const u8,
        buf_size: u32,
        to_addr: Option<DestTo>,
    ) -> Self {
        Self::UnsafeRef(SendZcUnsafeRefRec {
            fixed_fd,
            owner: Owner::Created,
            buf_const_u8,
            buf_size,
            to_addr,
        })
    }
    #[inline]
    pub(crate) fn entry(&self) -> io_uring::squeue::Entry {
        let (fixed_fd, buf_const_u8, buf_size, buf_index) = match self {
            Self::Fixed(f_rec) => (
                f_rec.fixed_fd,
                f_rec.buf_taken.buf_const_u8,
                2,
                //                f_rec.buf_taken.buf_size,
                None,
                //                Some(f_rec.buf_taken.buf_kernel_index),
            ),
            Self::UnsafeRef(u_rec) => (u_rec.fixed_fd, u_rec.buf_const_u8, u_rec.buf_size, None),
        };

        io_uring::opcode::SendZc::new(io_uring::types::Fixed(fixed_fd), buf_const_u8, buf_size)
            .buf_index(buf_index)
            .build() // todo dest_addr(), dest_addr_len(), flags(), zc_flags()
    }
    #[inline]
    pub(crate) fn owner(&self) -> Owner {
        match self {
            Self::Fixed(f) => f.owner.clone(),
            Self::UnsafeRef(r) => r.owner.clone(),
        }
    }
    #[inline]
    pub(crate) fn force_owner_kernel(&mut self) -> bool {
        match self {
            Self::Fixed(f) => f.owner = Owner::Kernel,
            Self::UnsafeRef(r) => r.owner = Owner::Kernel,
        }
        true
    }
}

/// When SendTo is sepcified, send(2) is turned into sendto(2)
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum DestTo {
    V4(DestToV4),
    V6(DestToV6),
}

/// IPv4 SendTo
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct DestToV4 {
    pub(crate) sockaddr: libc::sockaddr_in,
    pub(crate) socklen_t: libc::socklen_t,
}

/// IPv6 SendTo
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct DestToV6 {
    pub(crate) sockaddr: libc::sockaddr_in6,
    pub(crate) socklen_t: libc::socklen_t,
}
