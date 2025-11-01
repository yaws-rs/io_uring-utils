# io-uring-bufring

io_uring buffer ring mapping abstraction that can be used with io-uring-bearer or io-uring directly.

Note that this mapped ring of buffers does not handle data itself but you will need to pair
it with thing like [hugepage] or [anonymous mmap] where this crate maps the underlying memory
into a buffer ring that the linux kernel understands.

See the [bearer test](./src/ring_buf/ring_buf_test.rs) for an example.

[hugepage]: https://github.com/yaws-rs/ylibc/tree/main/hugepage
[anonymous_mmap]: https://github.com/yaws-rs/ylibc/tree/main/anonymous_mmap
