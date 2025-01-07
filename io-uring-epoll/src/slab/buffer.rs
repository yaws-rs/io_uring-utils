//! ProvideBuffer Slab Record

use crate::fd::HandledFd;
use crate::RawFd;

#[derive(Debug)]
pub struct ProvideBufferRec {
    bufs: Vec<Vec<u8>>,
    bgid: u16,
    bid: u16,
}

#[inline]
pub(crate) fn provide_buffer_rec() -> ProvideBufferRec {
    todo!()
}

#[inline]
fn entry(rec: &ProvideBufferRec) -> io_uring::squeue::Entry {
    todo!()
    //    io_uring::opcode::ProvideBuffers::new(
    //        buf, len, nbufs, bgid, bid
    //    ).build()
}
