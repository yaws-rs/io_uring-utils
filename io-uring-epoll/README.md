# io-uring-epoll

[![Discord chat][discord-badge]][discord-url]
[![Crates.io](https://img.shields.io/crates/v/io-uring-epoll.svg)](https://crates.io/crates/io-uring-epoll)
[![Docs](https://docs.rs/io-uring-epoll/badge.svg)](https://docs.rs/io-uring-epoll)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![License](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
![MSRV](https://img.shields.io/badge/MSRV-1.70.0-blue)

![meme what If I told you your epoll is in your io_uring](https://cdn.jsdelivr.net/gh/yaws-rs/io_uring-utils@main/io-uring-epoll/assets/meme_epoll_io_uring.jpg)

When your io_uring meets your epoll 🥰

Save system calls by setting file handle readiness checks especially in busy
eventloops that have a lot of on/off readiness activity via io_uring interface.

Please note that epoll is different to reqular poll and is only available on
Linux kernel.

Epoll itself has been in the Linux kernel around 20 years but io_uring has
recently added the EpollCtl OpCode support in order to bypass the need of
systerm calls to control it.

This is not a portable implementation given Windows I/O rings or MacOS doesn't
provide anything related with their relevant epoll implementations if any.

## Add

```ignore
cargo add io-uring-epoll
```

## Example

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
handler.add_fd(&handle_fd);

// Prepare a commit all changes into io_uring::SubmissionQueue
let handle_status = handler.prepare_submit().unwrap();

// async version is with submit()
handler.submit_and_wait(1).unwrap();

```

## License

Licensed under either of:

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

[discord-badge]: https://img.shields.io/discord/934761553952141402.svg?logo=discord
[discord-url]: https://discord.gg/rXVsmzhaZa