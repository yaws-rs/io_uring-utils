# io-uring-epoll

[![Discord chat][discord-badge]][discord-url]

![meme what If I told you your epoll is in your io_uring](https://cdn.jsdelivr.net/gh/yaws-rs/io_uring-utils@main/io-uring-epoll/assets/meme_epoll_io_uring.jpg)

When io_uring meets epoll it will save system calls for setting file handle readiness checks

Please note that epoll is different to reqular poll and is only available on Linux kernel

Epoll itself has been in the Linux kernel around 20 years but io_uring has recently added
the EpollCtl OpCode support in order to bypass the need of systerm calls to control it.

This is not a portable implementation given Windows I/O rings or MacOS doesn't provide
anything related.

```ignore
cargo add io-uring-epoll
```

```rust
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::os::fd::AsRawFd;
use io_uring_epoll::{HandledFd, EpollHandler};

// The 10 denotes power of two capacity to io_uring::IoUring
let mut handler = EpollHandler::new(10).expect("Unable to create EPoll Handler");

// This works with any impl that provides std::os::fd::AsRawFd impl
// In POSIX/UNIX-like it's just i32 file number or "fileno"
let listen = std::net::TcpListener::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0)).unwrap();

// Add the listen handle into EpollHandler
let mut handle_fd = HandledFd::new(listen.as_raw_fd());
let set_mask = handle_fd.set_in(true);
assert_eq!(set_mask, 1);
handler.add_fd(&handle_fd);

// Prepare a commit all changes into io_uring::SubmissionQueue
let handle_status = handler.prepare_submit().unwrap();
assert_eq!(handle_status.count_new(), 1);
assert_eq!(handle_status.count_changes(), 0);
assert_eq!(handle_status.count_empty(), 0);
assert_eq!(handle_status.errors().len(), 0);

// Take temp ref to io_uring::SubmissionQeueue
let submission  = handler.io_uring().submission();
assert_eq!(submission.len(), 1);
assert_eq!(submission.is_empty(), false);
assert_eq!(submission.dropped(), 0);
assert_eq!(submission.cq_overflow(), false);
assert_eq!(submission.is_full(), false);
drop(submission);

// async version is with submit()
handler.submit_and_wait(1).unwrap();

// Ensure that the kernel ate it
let submission  = handler.io_uring().submission();
assert_eq!(submission.len(), 0);
assert_eq!(submission.is_empty(), true);
assert_eq!(submission.dropped(), 0);
assert_eq!(submission.cq_overflow(), false);
assert_eq!(submission.is_full(), false);
drop(submission);



```

[discord-badge]: https://img.shields.io/discord/934761553952141402.svg?logo=discord
[discord-url]: https://discord.gg/rXVsmzhaZa