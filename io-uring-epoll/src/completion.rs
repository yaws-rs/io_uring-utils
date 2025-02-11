//! Competion types

use crate::slab::AcceptRec;
use crate::slab::EpollRec;
use crate::slab::FutexWaitRec;
use crate::slab::ProvideBuffersRec;
use crate::slab::SendZcRec;
use crate::slab::{RecvMultiRec, RecvRec};
use crate::Owner;

/// Completion types                      
#[derive(Clone, Debug)]
pub enum Completion {
    /// EpollCtl Completion               
    EpollEvent(EpollRec),
    /// Accept Completion                 
    Accept(AcceptRec),
    /// Provide Buffers
    ProvideBuffers(ProvideBuffersRec),
    /// Futex Wait
    FutexWait(FutexWaitRec),
    /// Recv
    Recv(RecvRec),
    /// RecvMulti
    RecvMulti(RecvMultiRec),
    /// SendZc
    SendZc(SendZcRec),
}

impl Completion {
    #[inline]
    pub(crate) fn entry(&self) -> io_uring::squeue::Entry {
        match self {
            Completion::Recv(r) => r.entry(),
            Completion::RecvMulti(r) => r.entry(),
            Completion::SendZc(r) => r.entry(),
            _ => todo!(),
        }
    }
    #[inline]
    pub(crate) fn owner(&self) -> Owner {
        match self {
            Self::Recv(ref recv) => recv.owner(),
            Self::RecvMulti(ref recv_multi) => recv_multi.owner(),
            Self::SendZc(ref send_zc) => send_zc.owner(),
            _ => todo!(),
        }
    }
    #[inline]
    pub(crate) fn force_owner_kernel(&mut self) -> bool {
        match self {
            Self::Recv(ref mut recv) => recv.force_owner_kernel(),
            Self::RecvMulti(ref mut recv_multi) => recv_multi.force_owner_kernel(),
            Self::SendZc(ref mut send_zc) => send_zc.force_owner_kernel(),
            _ => todo!(),
        }
    }
}

/// What to do with the submission record upon handling completion.
/// Used within handle_completions Fn Return                       
#[derive(Clone, Debug, PartialEq)]
pub enum SubmissionRecordStatus {
    /// Retain the original submsision record when it is needed to be retained.
    /// For example EpollCtl original Userdata must be retained in multishot mode.
    /// Downside is that care must be taken to clean up the associated sunmission record.
    Retain,
    /// Forget the associated submission record                                          
    /// For example Accept original record can be deleted upon compleiton after read.    
    /// Typically a new Accept submission is pushed without re-using any existing.       
    Forget,
}
