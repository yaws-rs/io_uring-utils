//! Competion types

use crate::slab::AcceptRec;
use crate::slab::EpollRec;
use crate::slab::ProvideBuffersRec;

/// Completion types                      
#[derive(Clone, Debug)]
pub enum Completion {
    /// EpollCtl Completion               
    EpollEvent(EpollRec),
    /// Accept Completion                 
    Accept(AcceptRec),
    /// Provide Buffers
    ProvideBuffers(ProvideBuffersRec),
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
