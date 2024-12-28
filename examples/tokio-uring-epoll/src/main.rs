//! This example is TOOD
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::mpsc;
use std::thread;

use io_uring_epoll::EpollUringHandler;
use tokio_uring::net::{TcpListener, TcpStream};

use std::os::fd::AsRawFd;
use std::os::fd::RawFd;

fn epoll_thread(tx: mpsc::Sender<RawFd>, rx: mpsc::Receiver<RawFd>) {
    let mut handler = EpollUringHandler::new(16).expect("Could not create handler");
}

fn main() {
    let (to_epoll, from_io) = mpsc::channel();
    let (to_io, from_epoll) = mpsc::channel();

    let epoll_thread = thread::spawn(move || {
        epoll_thread(to_io, from_io);
    });

    let listener =
        TcpListener::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0)).unwrap();
    let server_addr = listener.local_addr().unwrap();

    println!("Server listening at {:}", server_addr);

    let mut cycles = 0;

    loop {
        cycles += 1;

        let tick_tock = match cycles % 2 {
            0 => "tock _o_",
            _ => "tick _o/",
        };

        println!("[main] {}", tick_tock);
        thread::sleep(std::time::Duration::from_millis(1000));
    }
}
