#![warn(
    clippy::unwrap_used,
    missing_docs,
    rust_2018_idioms,
    unused_lifetimes,
    unused_qualifications
)]
#![doc = include_str!("../README.md")]

use io_uring::opcode::EpollCtl;
use io_uring::IoUring;

use std::collections::HashMap;
use std::collections::VecDeque;
use std::os::fd::RawFd;

/// Errors from the handler
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
    /// Something went yoinks in io_uring::IoRing::submit[_and_wait]
    Submission(String),
}

impl core::fmt::Display for EpollHandlerError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::IoUringCreate(s) => write!(f, "IoUring Create: {}", s),
            Self::EpollCreate1(s) => write!(f, "epoll_create1(): {}", s),
            Self::NotSupported => write!(
                f,
                "EpollCtl io_uring OpCode is not supported in your Kernel"
            ),
            Self::Probing(s) => write!(
                f,
                "Error whilst probing EpollCtl support from kernel: {}",
                s
            ),
            Self::Duplicate => write!(f, "The filehandle is already maped in. Possible duplicate?"),
            Self::Submission(s) => write!(f, "Submission: {}", s),
        }
    }
}

/// Manage the io_uring Submission and Completion Queues
/// related to EpollCtrl opcode in io_uring.
/// See io_epoll(7):
/// <https://man7.org/linux/man-pages/man7/epoll.7.html>
pub struct EpollHandler<'fd> {
    pub(crate) epfd: u32,
    pub(crate) io_uring: IoUring<io_uring::squeue::Entry, io_uring::cqueue::Entry>,
    pub(crate) in_flight: u32,
    pub(crate) fds: HashMap<i32, &'fd HandledFd>,
    pub(crate) submit_counter: u64,
}

impl<'fd> EpollHandler<'fd> {
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
    /// <https://docs.rs/io-uring/latest/io_uring/struct.Builder.html>
    ///
    /// Example:
    /// ```rust
    /// use io_uring::IoUring;
    /// use io_uring_epoll::EpollHandler;
    ///
    /// let mut iou: IoUring<io_uring::squeue::Entry, io_uring::cqueue::Entry>
    ///     = IoUring::builder()
    ///         .build(100)
    ///         .expect("Unable to build IoUring");
    ///
    /// EpollHandler::from_io_uring(iou).expect("Unable to create from io_uring Builder");
    /// ```    
    pub fn from_io_uring(
        iou: IoUring<io_uring::squeue::Entry, io_uring::cqueue::Entry>,
    ) -> Result<Self, EpollHandlerError> {
        let mut epoll_probe = io_uring::Probe::new();
        iou.submitter()
            .register_probe(&mut epoll_probe)
            .map_err(|e| EpollHandlerError::Probing(e.to_string()))?;

        #[allow(clippy::bool_comparison)]
        if epoll_probe.is_supported(EpollCtl::CODE) == false {
            return Err(EpollHandlerError::NotSupported);
        }

        // SAFETY: FFI no-data in
        let epfd = unsafe { libc::epoll_create1(0) };
        if epfd == -1 {
            // SAFETY: ffi no-data
            let errno = unsafe { libc::__errno_location() };
            return Err(EpollHandlerError::EpollCreate1(format!(
                "errno: {:?}",
                errno
            )));
        }

        Ok(Self {
            epfd: epfd as u32,
            io_uring: iou,
            in_flight: 0,
            submit_counter: 0,
            fds: HashMap::new(),
        })
    }
    /// Borrow the underlying io-uring::IoUring instance
    pub fn io_uring(&mut self) -> &mut IoUring<io_uring::squeue::Entry, io_uring::cqueue::Entry> {
        &mut self.io_uring
    }
    /// Add [`HandledFd`]
    /// Finally Commit changes to all changed HandledFds with [`EpollHandler::prepare_submit()`]
    pub fn add_fd(&mut self, handled_fd: &'fd HandledFd) -> Result<(), EpollHandlerError> {
        self.fds.insert(handled_fd.fd, handled_fd);
        Ok(())
    }
    /// This calls the underlying io_uring::IoUring::submit submitting all the staged commits
    pub fn submit(&self) -> Result<usize, EpollHandlerError> {
        self.io_uring
            .submit()
            .map_err(|e| EpollHandlerError::Submission(e.to_string()))
    }
    /// Same as submit but using io_uring::IoUring::submit_and_wait
    pub fn submit_and_wait(&self, want: usize) -> Result<usize, EpollHandlerError> {
        self.io_uring
            .submit_and_wait(want)
            .map_err(|e| EpollHandlerError::Submission(e.to_string()))
    }
    /// Stage commit of all changed [`HandledFd`] resources to submission queue
    /// This will make all changes within the underlying io_uring::squeue::SubmissionQueue
    /// Errors related on any individual staged Fds are available via FdCommitResults
    /// Use the [`Self::submit`] or [`Self::submit_and_wait`] to push the commits to kernel
    pub fn prepare_submit(&mut self) -> Result<FdCommitResults<'fd>, EpollHandlerError> {
        let mut fd_commit_results = FdCommitResults {
            new_commits: 0,
            change_commits: 0,
            no_change: 0,
            empty: 0,
            errors_on_submit: vec![],
        };

        // TODO: this is too long - split & refactor

        let iou = &mut self.io_uring;

        let mut s_queue = iou.submission();

        let mut updates: VecDeque<HandledFd> = VecDeque::new();

        for (_, handled_fd) in self.fds.iter() {
            let mut new_fd = (**handled_fd).clone();

            let mut commit_new: Option<i32> = None;
            let mut epoll_op = EPOLL_CTL_MOD;
            if handled_fd.wants.is_none() && handled_fd.committed.is_some() {
                epoll_op = EPOLL_CTL_DEL;
                commit_new = Some(0);
            }
            if let Some(fd_wants) = handled_fd.wants {
                match handled_fd.committed {
                    None => {
                        commit_new = Some(handled_fd.wants.unwrap_or(0));
                        epoll_op = EPOLL_CTL_ADD;
                        fd_commit_results.new_commits += 1;
                    }
                    Some(committed) => {
                        if committed != fd_wants {
                            commit_new = Some(fd_wants);
                            fd_commit_results.change_commits += 1;
                        } else {
                            fd_commit_results.no_change += 1;
                        }
                    }
                }
                if let Some(commit) = commit_new {
                    self.submit_counter += 1;

                    let epoll_event = libc::epoll_event {
                        events: commit as u32,
                        u64: self.submit_counter,
                    };

                    let uring_submission_rec = EpollCtl::new(
                        io_uring::types::Fixed(self.epfd),
                        io_uring::types::Fd(handled_fd.fd),
                        epoll_op,
                        std::ptr::addr_of!(epoll_event) as *const io_uring::types::epoll_event,
                    )
                    .build();

                    // SAFETY: op_code construct is type-valid and epoll_event is from libc Rust struct
                    let p_result = unsafe { s_queue.push(&uring_submission_rec) };
                    match p_result {
                        Err(e) => {
                            new_fd.error = Some(e.to_string());
                            fd_commit_results.errors_on_submit.push(handled_fd);
                        }
                        Ok(_) => {
                            new_fd.current_submission = Some(self.submit_counter);
                            self.in_flight += 1;
                        }
                    }
                }
            } else {
                // TODO: too long - refactor
                fd_commit_results.empty += 1;
            }

            // TODO: cursed
            if new_fd != **handled_fd {
                updates.push_back(new_fd);
            }
        }

        // TODO: This is a &&cursed &'mut &'fd situation ?!
        while let Some(update) = updates.pop_front() {
            let fd_get: Option<&&HandledFd> = self.fds.get(&update.fd);
            if let Some(unwrapped_fd) = fd_get {
                let mut pinned_mut: std::pin::Pin<&mut &HandledFd> = std::pin::pin!(*unwrapped_fd);
                let pinned_update: std::pin::Pin<&mut &HandledFd> = std::pin::pin!(&update);
                let _st = std::mem::replace(&mut pinned_mut, pinned_update);
            }
        }

        Ok(fd_commit_results)
    }
}

/// Create [`HandledFd::new(RawFd)`] and then add it to [`EpollHandler::add_fd()`]
/// 
/// *WARNING*: Your kernel may or may not have all wanted modes available
/// Consult your kernels epoll.h header to be sure and / or test if needed
#[derive(Debug, Clone, PartialEq)]
pub struct HandledFd {
    pub(crate) fd: RawFd,
    pub(crate) wants: Option<i32>,
    pub(crate) pending: Option<i32>,
    pub(crate) committed: Option<i32>,
    pub(crate) error: Option<String>,
    pub(crate) current_submission: Option<u64>,
}

const EPOLL_CTL_ADD: i32 = 1;
const EPOLL_CTL_DEL: i32 = 2;
const EPOLL_CTL_MOD: i32 = 3;

impl HandledFd {
    /// Create a new EpollHandler associated [`HandledFd`]
    /// Then add via [`EpollHandler::add_fd()`]
    pub fn new(fd: RawFd) -> Self {
        HandledFd {
            fd,
            wants: None,
            committed: None,
            current_submission: None,
            error: None,
            pending: None,
        }
    }
    /// Extract RawFd
    pub fn as_raw(&self) -> RawFd {
        self.fd
    }
    // All setters
    fn turn_on_or_off(&mut self, mask_in: i32, on_or_off: bool) -> i32 {
        let cur_wants: i32 = self.wants.unwrap_or(0);
        self.wants = match on_or_off {
            true => Some(cur_wants | mask_in),
            false => Some(cur_wants ^ mask_in),
        };
        self.wants.unwrap_or(0)
    }
    /// Set EPOLLIN per epoll.h in userspace On or Off
    /// Returns returns raw mask as to be sent to kernel
    /// Use [`EpollHandler::prepare_submit()`] after
    pub fn set_in(&mut self, on_or_off: bool) -> i32 {
        self.turn_on_or_off(libc::EPOLLIN, on_or_off)
    }
    /// EPOLLPRI
    pub fn set_pri(&mut self, on_or_off: bool) -> i32 {
        self.turn_on_or_off(libc::EPOLLPRI, on_or_off)
    }
    /// EPOLLOUT
    pub fn set_out(&mut self, on_or_off: bool) -> i32 {
        self.turn_on_or_off(libc::EPOLLOUT, on_or_off)
    }
    /// EPOLLERR
    pub fn set_err(&mut self, on_or_off: bool) -> i32 {
        self.turn_on_or_off(libc::EPOLLERR, on_or_off)
    }
    /// EPOLLHUP
    pub fn set_hup(&mut self, on_or_off: bool) -> i32 {
        self.turn_on_or_off(libc::EPOLLHUP, on_or_off)
    }
    /// EPOLLRDNORM
    pub fn set_rdnorm(&mut self, on_or_off: bool) -> i32 {
        self.turn_on_or_off(libc::EPOLLRDNORM, on_or_off)
    }
    /// EPOLLRDBAND
    pub fn set_rdband(&mut self, on_or_off: bool) -> i32 {
        self.turn_on_or_off(libc::EPOLLRDBAND, on_or_off)
    }
    /// EPOLLWRNORM
    pub fn set_wrnorm(&mut self, on_or_off: bool) -> i32 {
        self.turn_on_or_off(libc::EPOLLWRNORM, on_or_off)
    }
    /// EPOLLWRBAND per epoll.h userspace On or Off
    pub fn set_wrband(&mut self, on_or_off: bool) -> i32 {
        self.turn_on_or_off(libc::EPOLLWRBAND, on_or_off)
    }
    /// EPOLLMSG
    pub fn set_msg(&mut self, on_or_off: bool) -> i32 {
        self.turn_on_or_off(libc::EPOLLMSG, on_or_off)
    }
    /// EPOLLRDHUP
    pub fn set_rdhup(&mut self, on_or_off: bool) -> i32 {
        self.turn_on_or_off(libc::EPOLLRDHUP, on_or_off)
    }
    /// EPOLLWAKEUP
    pub fn set_wakeup(&mut self, on_or_off: bool) -> i32 {
        self.turn_on_or_off(libc::EPOLLWAKEUP, on_or_off)
    }
    /// EPOLLONESHOT
    pub fn set_oneshot(&mut self, on_or_off: bool) -> i32 {
        self.turn_on_or_off(libc::EPOLLONESHOT, on_or_off)
    }
    /// EPOLLET
    pub fn set_et(&mut self, on_or_off: bool) -> i32 {
        self.turn_on_or_off(libc::EPOLLET, on_or_off)
    }
    /// Get the raw u32 Epoll event mask as set in userspace
    /// This may not have been sent and may be pending send or not committed
    /// Use [`EpollHandler::prepare_submit()`] after
    pub fn get_mask_raw(&mut self) -> Option<i32> {
        self.wants
    }
    /// Set the raw u32 Epoll event mask in the userspace
    /// *WARNING*: Ensure this is valid per epoll.h of your kernel
    /// Use [`EpollHandler::prepare_submit()`] after
    pub fn set_mask_raw(&mut self, mask: i32) {
        self.wants = Some(mask);
    }
    /// Get the pending eq u32 Epoll
    /// This may not be committed into kernel yet use get_committed to check
    /// This will be none if there is no pending change or it has not been sent
    /// Use [`EpollHandler::prepare_submit()`] after
    pub fn get_pending(&self) -> Option<i32> {
        self.pending
    }
    // /// Get the committed raw u32 Epoll event mask if any
    // /// This represents the state that has been confirmed by the kernel
    // /// Use [`EpollHandler::commit()`] to commit all pending changes if any
    // TODO: handle this internally
    //pub fn get_committed(&self) -> Option<i32> {
    //    self.committed
    //}
}

/// Represents Submission queue results as described in [`EpollHandler::commit()`]
#[derive(Debug)]
pub struct FdCommitResults<'fd> {
    pub(crate) new_commits: u32,
    pub(crate) change_commits: u32,
    pub(crate) no_change: u32,
    pub(crate) empty: u32,
    pub(crate) errors_on_submit: Vec<&'fd HandledFd>,
}

impl<'fd> FdCommitResults<'fd> {
    /// How many new EPOLL_ADD entries in SubmissionQueue
    pub fn count_new(&self) -> u32 {
        self.new_commits
    }
    /// How many nw EPOLL_MOD entries in SubmissionQueue
    pub fn count_changes(&self) -> u32 {
        self.change_commits
    }
    /// How many handles did not see any changes
    pub fn count_no_changes(&self) -> u32 {
        self.no_change
    }
    /// How many handles are empty
    pub fn count_empty(&self) -> u32 {
        self.empty
    }
    /// How many updates gave error upon submission
    pub fn errors(&'fd self) -> &'fd Vec<&'fd HandledFd> {
        &self.errors_on_submit
    }
}

#[cfg(test)]
mod test {
    use super::HandledFd;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
    use std::os::fd::AsRawFd;

    fn handle_fd() -> HandledFd {
        let s =
            TcpListener::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0)).unwrap();
        HandledFd::new(s.as_raw_fd())
    }

    #[test]
    fn mask_fd_inouts() {
        let mut fd = handle_fd();
        fd.set_in(true);
        assert_eq!(fd.set_out(true), 5);
        assert_eq!(fd.set_in(false), 4);
        assert_eq!(fd.set_out(false), 0);
    }
    #[test]
    fn mask_fd_in() {
        let mut fd = handle_fd();
        assert_eq!(fd.set_in(true), 1);
        assert_eq!(fd.set_in(false), 0);
    }
    #[test]
    fn mask_fd_pri() {
        let mut fd = handle_fd();
        assert_eq!(fd.set_pri(true), 2);
        assert_eq!(fd.set_pri(false), 0);
    }
    #[test]
    fn mask_fd_out() {
        let mut fd = handle_fd();
        assert_eq!(fd.set_out(true), 4);
        assert_eq!(fd.set_out(false), 0);
    }
    #[test]
    fn mask_fd_err() {
        let mut fd = handle_fd();
        assert_eq!(fd.set_err(true), 8);
        assert_eq!(fd.set_err(false), 0);
    }
    #[test]
    fn mask_fd_hup() {
        let mut fd = handle_fd();
        assert_eq!(fd.set_hup(true), 0x00000010);
        assert_eq!(fd.set_hup(false), 0);
    }
    #[test]
    fn mask_fd_rdnorm() {
        let mut fd = handle_fd();
        assert_eq!(fd.set_rdnorm(true), 0x00000040);
        assert_eq!(fd.set_rdnorm(false), 0);
    }
    #[test]
    fn mask_fd_rdband() {
        let mut fd = handle_fd();
        assert_eq!(fd.set_rdband(true), 0x00000080);
        assert_eq!(fd.set_rdband(false), 0);
    }
    #[test]
    fn mask_fd_wrnorm() {
        let mut fd = handle_fd();
        assert_eq!(fd.set_wrnorm(true), 0x00000100);
        assert_eq!(fd.set_wrnorm(false), 0);
    }
    #[test]
    fn mask_fd_wrband() {
        let mut fd = handle_fd();
        assert_eq!(fd.set_wrband(true), 0x00000200);
        assert_eq!(fd.set_wrband(false), 0);
    }
    #[test]
    fn mask_fd_msg() {
        let mut fd = handle_fd();
        assert_eq!(fd.set_msg(true), 0x00000400);
        assert_eq!(fd.set_msg(false), 0);
    }
    #[test]
    fn mask_fd_rdhup() {
        let mut fd = handle_fd();
        assert_eq!(fd.set_rdhup(true), 0x00002000);
        assert_eq!(fd.set_rdhup(false), 0);
    }
    #[test]
    fn mask_fd_wakeup() {
        let mut fd = handle_fd();
        assert_eq!(fd.set_wakeup(true), 0x20000000);
        assert_eq!(fd.set_wakeup(false), 0);
    }
    #[test]
    fn mask_fd_oneshot() {
        let mut fd = handle_fd();
        assert_eq!(fd.set_oneshot(true), 0x40000000);
        assert_eq!(fd.set_oneshot(false), 0);
    }
    #[test]
    fn mask_fd_et() {
        let mut fd = handle_fd();
        assert_eq!(fd.set_et(true) as u32, 0x80000000);
        assert_eq!(fd.set_et(false), 0);
    }
}
