#![warn(
    clippy::unwrap_used,
    missing_docs,
    rust_2018_idioms,
    unused_lifetimes,
    unused_qualifications
)]
#![doc = include_str!("../README.md")]

use io_uring::IoUring;
use io_uring::opcode::EpollCtl;
use std::os::fd::{AsRawFd, RawFd};
use std::collections::VecDeque;

#[derive(Debug)]
pub enum EpollHandlerError {
    /// Error creating IoUring instance
    IoUringCreate(String),
    /// Error creating epoll handle in Kernel
    EpollCreate1(String),
    /// EpollCtl OpCode is not supported in your kernel
    NotSupported,
    /// Error probing the support of EpollCtl from kernel
    Probing(String),
    /// The Fd is already in the handler and would override existing
    Duplicate,
}

impl core::fmt::Display for EpollHandlerError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::IoUringCreate(s) => write!(f, "IoUring Create: {}", s),
            Self::EpollCreate1(s) => write!(f, "epoll_create1(): {}", s),
            Self::NotSupported => write!(f, "EpollCtl io_uring OpCode is not supported in your Kernel"),
            Self::Probing(s) => write!(f, "Error whilst probing EpollCtl support from kernel: {}", s),
        }
    }
}

pub struct EpollHandler {
    pub(crate) epfd: u32,
    pub(crate) io_uring: IoUring<io_uring::squeue::Entry, io_uring::cqueue::Entry>,
    pub(crate) in_flight: u32,
    pub(crate) backlog: VecDeque<io_uring::squeue::Entry>,
    pub(crate) fds: HashMap<u32, HandledFd>,
    pub(crate) submit_counter: u64,
}

impl EpollHandler {
    /// Create a new handler with new io-uring::IoUring
    ///
    /// capacity must be power of two as per io-uring documentation
    /// ```rust
    /// use io_uring_epoll::EpollHandler;
    ///
    /// EpollHandler::new(10).expect("Unable to create EPoll Handler");
    /// ```    
    pub fn new(capacity: u32) -> Result<Self, EpollHandlerError> {
        let iou: IoUring<io_uring::squeue::Entry, io_uring::cqueue::Entry> = IoUring::builder()
            .build(capacity)
            .map_err(|e| EpollHandlerError::IoUringCreate(e.to_string()))?;

        Self::from_io_uring(iou)
    }
    /// Create a new handler from an existing io-uring::IoUring builder
    /// To construct a custom IoUring see io-uring Builder:
    /// https://docs.rs/io-uring/latest/io_uring/struct.Builder.html
    ///
    /// Example:
    /// ```rust
    ///
    /// let mut iou: IoUring<io_uring::squeue::Entry, io_uring::cqueue::Entry>
    ///     = IoUring::builder()
    ///         .build(cap)
    ///         .expect("Unable to build IoUring");
    ///
    /// EpollHandler::from_io_uring(iou).expect("Unable to create from io_uring Builder");
    /// ```    
    pub fn from_io_uring(iou: IoUring<io_uring::squeue::Entry, io_uring::cqueue::Entry>) -> Result<Self, EpollHandlerError> {

        let mut epoll_probe = io_uring::Probe::new();
        iou.submitter().register_probe(&mut epoll_probe).map_err(|e| EpollHandlerError::Probing(e.to_string()))?;

        if epoll_probe.is_supported(io_uring::opcode::EpollCtl::CODE) == false {
            return Err(EpollHandlerError::NotSupported);
        }
        
        // SAFETY: FFI no-data in
        let epfd = unsafe { libc::epoll_create1(0) };
        if epfd == -1 {
            // SAFETY: ffi no-data
            let errno = unsafe { libc::__errno_location() };
            return Err(EpollHandlerError::EpollCreate1(format!("errno: {:?}", errno)));
        }

        Ok( Self { epfd: epfd as u32, io_uring: iou, in_flight: 0, backlog: VecDeque::new(), submit_counter: u64 } )
    }
    /// Add [`std::os::fd::RawFd`] to Handler
    /// After this set the states via the created [`HandledFd`] and then
    /// commit changes to all changed HandledFds with [`EpollHandler::commit`]
    /// To remove call [`HandledFd::drop`] or automatically upon drop
    pub fn add_raw_fd(&mut self, fd: RawFd) -> Result<HandledFd, EpollHandlerError> {
        let handled_fd = HandledFd { fd, wants: None, committed: None };

        if self.fds.get(fd) {
            return Err(EpollHandlerError::Duplicate);
        }

        self.fds.set(fd, handled_fd);
        
        Ok(HandledFd)
    }
    /// Commit all changed [`HandledFd`] to kernel
    /// This will submit all changes to SubmissionQueue and may not be immediately set
    /// The setting/s may also fail and leave hanging in pending change over commit
    /// Submission errors related on any individual Fds is available via FdCommitResults
    pub fn commit(&mut self) -> Result<FdCommitResults, EpollHandlerError> {
        let mut fd_commit_results = FdCommitResults { new_commits: 0, change_commits: 0, no_change: 0, empty: 0, errors_on_submit: Vec<&HandledFd> };

        let mut iou = &mut self.iou;
        
        let mut s_queue = iou.submission();
        
        self.fds.as_ref().for_each(|fd| {
            let mut commit_new: Option<u32> = None;
            let mut epoll_op = EPOLL_CTL_MOD;
            if let Some(fd_wants) = fd.wants {
                match fd.committed {
                    None => {
                        commit_new = fd_wants,
                        epoll_op = EPOLL_CTL_ADD;
                        fd_commit_results.new_commits += 1;
                    },
                    Some(committed) => {
                        if committed != fd_wants {
                            commit_new = fd_wants;
                            fd_commit_results.change_commits += 1;
                        }
                        else {
                            fd_commit_results.no_change += 1;
                        }
                    },
                }
                else {
                    fd_commit_results.empty += 1;
                }
                if let Some(commit) = commit_new {

                    self.submit_counter+=1;
                    
                    let epoll_event = libc::epoll_event { events: fd.wanted, u64: self.submit_counter };

                    let uring_submission_rec = EpollCtl::new(
                        io_uring::types::Fixed(epfd),
                        io_uring::types::Fd(fd),
                        epoll_op,
                        std::ptr::addr_of!(epoll_event) as *const io_uring::types::epoll_event
                    ).build();

                    // SAFETY: op_code is valid and epoll_event is from libc Rust struct
                    let p_result = unsafe { s_queue.push(&op_code) };
                    match p_result {
                        Err(e) => {
                            fd.error = e.to_string();
                            push(fd_commit_results.errors_on_submit, &fd);
                        },
                        Ok(_) => {
                            fd.current_submission = self.submit_counter;
                        },
                    }
                }
                
            }
        });
    }
}

/// Create it via [`EpollHandler::add_raw_fd`]
/// WARNING: Your kernel may or may not have all wanted modes available
/// Consult your kernels epoll.h header to be sure and / or test if needed
#[derive(Debug)]
pub struct HandledFd {
    pub(crate) fd: RawFd,
    pub(crate) wants: Option<u32>,
    pub(crate) pending: Option<u32>,
    pub(crate) committed: Option<u32>,
    pub(crate) error: Option<String>,
    pub(crate) current_submission: Option<u64>,
}

const EPOLL_CTL_ADD: i32 = 1
const EPOLL_CTL_DEL: i32 = 2
const EPOLL_CTL_MOD: i32 = 3

impl HandledFd {
    /// Set EPOLLIN per epoll.h in userspace On or Off
    /// Use [`EpollHandler::commit`] to commit all pending changes after        
    pub fn in(&mut self, on_or_off: bool) -> bool {
        match on_or_off {
            true => self.wants |= libc::EPOLLIN,
            false => self.wants ^= libc::EPOLLIN,
        }
        self.wants & libc::EPOLLIN
    }
    /// Set EPOLLPRI per epoll.h in userspace On or Off
    /// Use [`EpollHandler::commit`] to commit all pending changes after        
    pub fn pri(&mut self, on_or_off: bool) -> bool {
        match on_or_off {
            true => self.wants |= libc::EPOLLPRI,
            false => self.wants ^= libc::EPOLLPRI,
        }
        self.wants & libc::EPOLLPRI
    }
    /// Set EPOLLOUT per epoll.h in userspace On or Off
    /// Use [`EpollHandler::commit`] to commit all pending changes after        
    pub fn out(&mut self, on_or_off: bool) {
        match on_or_off {
            true => self.wants |= libc::EPOLLOUT,
            false => self.wants ^= libc::EPOLLOUT,
        }
    }
    /// Set EPOLLHUP per epoll.h in userspace On or Off
    /// Use [`EpollHandler::commit`] to commit all pending changes after        
    pub fn hup(&mut self, on_or_off: bool) {
        match on_or_off {
            true => self.wants |= libc::EPOLLHUP,
            false => self.wants ^= libc::EPOLLHUP,
        }
    }
    /// Set EPOLLRDNORM per epoll.h userspace On or Off
    /// Use [`EpollHandler::commit`] to commit all pending changes after        
    pub fn rdnorm(&mut self, on_or_off: bool) {
        match on_or_off {
            true => self.wants |= libc::EPOLLRDNORM,
            false => self.wants ^= libc::EPOLLRDNORM,
        }
    }
    /// Set EPOLLRDBAND per epoll.h. userspace On or Off
    /// Use [`EpollHandler::commit`] to commit all pending changes after        
    pub fn rdband(&mut self, on_or_off: bool) {
        match on_or_off {
            true => self.wants |= libc::EPOLLRDBAND,
            false => self.wants ^= libc::EPOLLRDBAND,
        }
    }
    /// Set EPOLLWRNORM per epoll.h userspace On or Off
    /// Use [`EpollHandler::commit`] to commit all pending changes after        
    pub fn wrnorm(&mut self, on_or_off: bool) {
        match on_or_off {
            true => self.wants |= libc::EPOLLWRNORM,
            false => self.wants ^= libc::EPOLLWRNORM,
        }
    }
    /// Set EPOLLWRBAND per epoll.h userspace On or Off
    /// Use [`EpollHandler::commit`] to commit all pending changes after        
    pub fn wrband(&mut self, on_or_off: bool) {
        match on_or_off {
            true => self.wants |= libc::EPOLLWRBAND,
            false => self.wants ^= libc::EPOLLWRBAND,
        }

    }
    /// Set EPOLLMSG per epoll.h userspace On or Off
    /// Use [`EpollHandler::commit`] to commit all pending changes after        
    pub fn msg(&mut self, on_or_off: bool) {
        match on_or_off {
            true => self.wants |= libc::EPOLLMSG,
            false => self.wants ^= libc::EPOLLMSG,
        }
    }
    /// Set EPOLLRDHUP per epoll.h userspace On or Off
    /// Use [`EpollHandler::commit`] to commit all pending changes after        
    pub fn rdhup(&self, on_or_off: bool) {
        match on_or_off {
            true => self.wants |= libc::EPOLLRDHUP,
            false => self.wants ^= libc::EPOLLRDHUP,
        }
    }
    /// Set EPOLLWAKEUP per epoll.h userspace On or Off
    /// Use [`EpollHandler::commit`] to commit all pending changes after        
    pub fn wakeup(&self, on_or_off: bool) {
        set on_or_off {
            true => self.wants |= libc::EPOLLWAKEUP,
            false => self.wants ^= libc::EPOLLWAKEUP,
        }
    }
    /// Set EPOLLONESHOT per epoll.h userspace On or Off
    /// Use [`EpollHandler::commit`] to commit all pending changes after        
    pub fn oneshot(&self, on_or_off: bool) {
        match on_or_off {
            true => self.wants |= libc::EPOLLONESHOT,
            false => self.wants ^= libc::EPOLLONESHOT,
        }
    }
    /// Set EPOLLET per epoll.h userspace On  or Off
    /// Use [`EpollHandler::commit`] to commit all pending changes after    
    pub fn et(&self, on_or_off: bool) {
        match on_or_off {
            true => self.wants |= libc::EPOLLET,
            false => self.wants ^= libc::EPOLLET,
        }
    }
    /// Get the raw u32 Epoll event mask as set in userspace
    /// This may not be yet sent pending or committed into kernel
    /// Use [`EpollHandler::commit`] to commit all pending changes if any
    pub fn get_mask_raw(&mut self) -> u32 {
        self.wants
    }
    /// Set the raw u32 Epoll event mask in the userspace
    /// *WARNING*: Ensure this is valid per epoll.h of your kernel
    /// Use [`EpollHandler::commit`] to commit all pending changes after
    pub fn set_mask_raw(&mut self, mask: u32) -> u32 {
        self.wants = mask;
        self.wants
    }
    /// Get the pending eq u32 Epoll
    /// This may not be committed into kernel yet use get_committed to check
    /// This will be none if there is no pending change or it has not been sent
    /// Use [`EpollHandler::commit`] to re-commit all pending changes if any
    pub fn get_pending(&self) -> Option<u32> {
        self.pending
    }
    /// Get the committed raw u32 Epoll event mask if any
    /// This represents the state that has been confirmed by the kernel
    /// Use [`EpollHandler::commit`] to commit all pending changes if any
    pub fn get_committed(&self) -> Option<u32> {
        self.committed
    }
}

