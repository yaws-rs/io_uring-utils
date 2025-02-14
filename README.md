# io_uring Utils

Higher level abstractions to deal with io_uring in various contextes.

This builds on top of [io-uring](https://crates.io/crates/io-uring) crate.

# Crates

| Crate             | Description                                         |
| :---              | :---                                                |
| [io-uring-bearer] | Bearer for the io_uring                             |
| [io-uring-epoll]  | EpollCtl OpCode implementation and utilities        |
| [io-uring-opcode] | OpCode extension trait and harmonized Error         |
| [io-uring-fd]     | Associated filehandle types                         |
| [io-uring-owner]  | Ownership semantics                                 |
| [io-uring-probe]  | Probing (WIP)                                       |

[io-uring-bearer]: ./io-uring-bearer
[io-uring-epoll]: ./io-uring-epoll
[io-uring-opcode]: ./io-uring-opcode
[io-uring-fd]: ./io-uring-fd
[io-uring-owner]: ./io-uring-owner
[io-uring-prob]: ./io-uring-probe
