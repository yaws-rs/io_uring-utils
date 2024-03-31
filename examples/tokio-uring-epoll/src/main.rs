use std::net::ToSocketAddrs;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;

use io_uring_epoll::EpollHandler;
use tokio_uring::net::TcpStream;

use std::os::fd::AsRawFd;
use std::os::fd::RawFd;

fn epoll_thread(tx: u32, rx: u32) {
    let mut handler = EpollHandler::new(4).expect("Could not create handler");
}

fn main() {
    let (to_epoll, from_io) = mpsc::channel();
    let (to_io, from_epoll) = mpsc::channel();

    let epoll_thread = thread::spawn(move || {
        epoll_thread(to_io, from_io);
    });

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let server_addr = listener.local_addr().unwrap();
}
