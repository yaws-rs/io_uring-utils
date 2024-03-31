# io-uring-epoll

[![Discord chat][discord-badge]][discord-url]

```ignore
cargo add io-uring-epoll
```

```rust
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::os::fd::AsRawFd;
use io_uring_epoll::{HandledFd, EpollHandler};

let mut handler = EpollHandler::new(10).expect("Unable to create EPoll Handler");

let listen = std::net::TcpListener::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0)).unwrap();

let mut handle_fd = HandledFd::new(listen.as_raw_fd());

let set_mask = handle_fd.set_in(true);

assert_eq!(set_mask, 1);

handler.add_fd(&handle_fd);

handler.commit().unwrap();

```

See examples directory.

[discord-badge]: https://img.shields.io/discord/934761553952141402.svg?logo=discord
[discord-url]: https://discord.gg/rXVsmzhaZa