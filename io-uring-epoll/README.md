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

See [Examples](./examples) directory for the different use-cases.

## License

Licensed under either of:

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

[discord-badge]: https://img.shields.io/discord/934761553952141402.svg?logo=discord
[discord-url]: https://discord.gg/rXVsmzhaZa