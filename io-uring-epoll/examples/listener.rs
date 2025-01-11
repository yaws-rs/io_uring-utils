use io_uring_epoll::Completion;
use io_uring_epoll::EpollUringHandler;
use io_uring_epoll::HandledFd;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::os::fd::AsRawFd;

fn main() {
    // Creates new Epoll Uring handler with size 16 and capacity 16 for both fd and epoll registers
    let mut handler = EpollUringHandler::new(16, 16, 16).expect("Unable to create EPoll Handler");

    // This works with any impl that provides std::os::fd::AsRawFd impl
    // In POSIX/UNIX-like it's just i32 file number or "fileno"
    // The user is responsible of keeping the filehandle "alive" and should use OwnedFd as base
    let listen =
        std::net::TcpListener::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0))
            .unwrap();

    // Add the listen handle into EpollHandler
    let mut handle_fd = HandledFd::from_raw(listen.as_raw_fd());
    let set_mask = handle_fd.set_in(true);
    assert_eq!(set_mask, 1);
    handler.commit_fd(&handle_fd).unwrap();

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

    // Completion may take some time
    {
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

    #[derive(Debug)]
    struct UserData {
        e: u32,
    }

    let mut user = UserData { e: 0 };

    // The underlying UringHandler is accessible
    let uring_handler = handler.uring_handler();

    // This is unsafe because forgetting the original submission record
    // can lead to UB where the kernel still refers to it's address later.
    /*
    unsafe { uring_handler.handle_completions(&mut user, |user, entry, rec| {
        match rec {
            Completion::EpollEvent(e) => {
                user.e += 1;
                println!("EpollEvent => {:?} Entry => {:?}", e, entry);
                // SAFETY: We retain the record and will not move it
                SubmissionRecordStatus::Retain
            }
            _ => todo!(),
        }
    }) };
    */

    // This is safe since we are not touching the original submission record.
    uring_handler
        .completions(&mut user, |user, entry, rec| match rec {
            Completion::EpollEvent(e) => {
                user.e += 1;
                println!("EpollEvent => {:?} Entry => {:?}", e, entry);
            }
            _ => todo!(),
        })
        .unwrap();
}
