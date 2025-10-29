//! Submission related types

use crate::error::UringBearerError;
use io_uring::squeue::Flags as IoUringFlags;

/// See Linux io_uring_sqe_set_flags(3) for IOSQE_ASYNC for the respective documentation.
/// These flags may be set for each submission queue entry. By default all flags are Off.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct SubmissionFlags {
    pub(crate) bits: u8,
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

/// Indicate whether the resulting filehandle upon completion of the submission
/// should be registered / associated as "fixed" to the io_uring bearer or not.
///
/// Registering the filahandle here can avoid additional overhead of registering it later
/// separately within the io_uring context.
///
/// Registered "fixed" io_uring filehandles are not available in non-io_uring context and
/// start from zero.
///
/// When registering the filehandle within io_uring there has to be free space within the
/// initialized map of filehandles done prior and when manually targeting a fixed slot
/// it must be free and ready to be assigned.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TargetFd {
    /// Regular non-io_uring registered filehandle. Useful for non-io_uring uses.
    Unregistered,
    /// Auto-assigned fixed slot io_uring registered or "fixed" filehandle.
    AutoRegistered,
    /// Manually assigned fixed slot io_uring registered or "fixed" filehandle.
    ManualRegistered(u32),
}
