use io_uring_epoll::{EpollHandler, HandledFd};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::os::fd::AsRawFd;

fn main() {
    // The 10 denotes power of two capacity to io_uring::IoUring
    let mut handler = EpollHandler::new(10).expect("Unable to create EPoll Handler");

    // This works with any impl that provides std::os::fd::AsRawFd impl
    // In POSIX/UNIX-like it's just i32 file number or "fileno"
    let listen =
        std::net::TcpListener::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0))
            .unwrap();

    // Add the listen handle into EpollHandler
    let mut handle_fd = HandledFd::new(listen.as_raw_fd());
    let set_mask = handle_fd.set_in(true);
    assert_eq!(set_mask, 1);
    handler.add_fd(&handle_fd).unwrap();

    // Prepare a commit all changes into io_uring::SubmissionQueue
    let handle_status = handler.prepare_submit().unwrap();
    assert_eq!(handle_status.count_new(), 1);
    assert_eq!(handle_status.count_changes(), 0);
    assert_eq!(handle_status.count_empty(), 0);
    assert_eq!(handle_status.errors().len(), 0);

    // Take temp ref to io_uring::SubmissionQeueue
    let submission = handler.io_uring().submission();
    assert_eq!(submission.len(), 1);
    assert_eq!(submission.is_empty(), false);
    assert_eq!(submission.dropped(), 0);
    assert_eq!(submission.cq_overflow(), false);
    assert_eq!(submission.is_full(), false);
    drop(submission);

    // async version is with submit()
    handler.submit_and_wait(1).unwrap();

    // Ensure that the kernel ate it
    let submission = handler.io_uring().submission();
    assert_eq!(submission.len(), 0);
    assert_eq!(submission.is_empty(), true);
    assert_eq!(submission.dropped(), 0);
    assert_eq!(submission.cq_overflow(), false);
    assert_eq!(submission.is_full(), false);
    drop(submission);

    let c_queue = handler.io_uring().completion();
    let mut c_attempts = 0;
    loop {
        if c_queue.is_empty() == false {
            assert_eq!(c_queue.len(), 1);
            break;
        }
        if c_attempts == 10 {
            panic!("Took more than 100 ms - completion never finished?");
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
        c_attempts += 1;
    }
}
