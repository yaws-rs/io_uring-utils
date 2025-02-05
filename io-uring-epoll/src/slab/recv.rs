//! Recv Slab records

use crate::Owner;

use crate::slab::buffer::TakenBuffer;

/// Recv Record
#[derive(Clone, Debug)]
pub struct RecvRec {
    fixed_fd: u32,
    owner: Owner,
    buf_taken: TakenBuffer,
}

impl RecvRec {
    #[inline]
    pub(crate) fn new(fixed_fd: u32, buf_taken: TakenBuffer) -> Self {
        RecvRec {
            fixed_fd,
            owner: Owner::Created,
            buf_taken,
        }
    }
    #[inline]
    pub(crate) fn entry(&self) -> io_uring::squeue::Entry {
        io_uring::opcode::Recv::new(
            io_uring::types::Fixed(self.fixed_fd),
            self.buf_taken.buf_mut_u8,
            self.buf_taken.buf_size,
        )
        .build()
    }
    #[inline]
    pub(crate) fn owner(&self) -> Owner {
        self.owner.clone()
    }
    #[inline]
    pub(crate) fn force_owner_kernel(&mut self) -> bool {
        self.owner = Owner::Kernel;
        true
    }
}

/// RecvMulti Record
#[derive(Clone, Debug)]
pub struct RecvMultiRec {
    fixed_fd: u32,
    owner: Owner,
    buf_grp_id: u16,
}

impl RecvMultiRec {
    #[inline]
    pub(crate) fn new(fixed_fd: u32, buf_grp_id: u16) -> Self {
        Self {
            fixed_fd,
            buf_grp_id,
            owner: Owner::Created,
        }
    }
    #[inline]
    pub(crate) fn entry(&self) -> io_uring::squeue::Entry {
        io_uring::opcode::RecvMulti::new(io_uring::types::Fixed(self.fixed_fd), self.buf_grp_id)
            .build()
    }
    #[inline]
    pub(crate) fn owner(&self) -> Owner {
        self.owner.clone()
    }
    #[inline]
    pub(crate) fn force_owner_kernel(&mut self) -> bool {
        self.owner = Owner::Kernel;
        true
    }
}
