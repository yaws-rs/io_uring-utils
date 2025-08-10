//! Submission related types

use crate::error::UringBearerError;
use io_uring::squeue::Flags as IoUringFlags;

/// See Linux io_uring_sqe_set_flags(3) for IOSQE_ASYNC for the respective documentation.
/// These flags may be set for each submission queue entry. By default all flags are Off.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SubmissionFlags {
    pub(crate) bits: u8,
    /*    pub(crate) fl_io_drain: bool,
    pub(crate) fl_io_link: bool,
    pub(crate) fl_io_hard_link: bool,
    pub(crate) fl_async: bool,
    pub(crate) fl_buffer_select: bool,
    pub(crate) fl_skip_success: bool, */
}

impl SubmissionFlags {
    /// Submission of this entry will not start before all the prior entries are completed.
    /// Submissions after this entry will not complete before this entry is completed.
    #[inline]
    pub fn on_io_drain(mut self) -> Self {
        self.bits |= IoUringFlags::IO_DRAIN.bits();
        self
    }
    /// Submission is linked to the next after this so it will not start before this is completed.
    #[inline]
    pub fn on_io_link(mut self) -> Self {
        self.bits |= IoUringFlags::IO_LINK.bits();
        self
    }
    /// Same as on_io_link but does not sever regardless of completion result.
    // TODO(doc): what does this actually mean - e.g. will next one get cancelled if this fails ? etc.
    #[inline]
    pub fn on_io_hard_link(mut self) -> Self {
        self.bits |= IoUringFlags::IO_HARDLINK.bits();
        self
    }
    /// Issue direct async submission over non-blocking directly. See Linux io_uring_sqe_set_flags(3).
    #[inline]
    pub fn on_async(mut self) -> Self {
        self.bits |= IoUringFlags::ASYNC.bits();
        self
    }
    /// Signal to kernel to select a buffer from a given group id.
    #[inline]
    pub fn on_buffer_select(mut self) -> Self {
        self.bits |= IoUringFlags::BUFFER_SELECT.bits();
        self
    }
    /// Skip all the completion events when sucessfull out of this submission.
    #[inline]
    pub fn on_skip_success(mut self) -> Self {
        self.bits |= IoUringFlags::SKIP_SUCCESS.bits();
        self
    }
    /// Convert to [`io-uring::squeue::Flags`]
    #[inline]
    pub fn to_io_uring_flags(&self) -> Result<io_uring::squeue::Flags, UringBearerError> {
        match io_uring::squeue::Flags::from_bits(self.bits) {
            None => Err(UringBearerError::SubmissionFlags),
            Some(ret) => Ok(ret),
        }
    }
}
