use std::net::TcpListener;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::os::fd::AsRawFd;

use ysockaddr::YSockAddrR;

use io_uring_bearer::Completion;
use io_uring_bearer::UringBearer;

use capacity::{Capacity, Setting};
use io_uring_bearer::BearerCapacityKind;

use io_uring_op_connect::Connect;
use io_uring_opcode::OpExtConnect;

#[derive(Clone, Debug)]
pub struct MyCapacity;

impl Setting<BearerCapacityKind> for MyCapacity {
    fn setting(&self, v: &BearerCapacityKind) -> usize {
        match v {
            BearerCapacityKind::CoreQueue => 1,
            BearerCapacityKind::RegisteredFd => 1,
            BearerCapacityKind::PendingCompletions => 1,
            BearerCapacityKind::Buffers => 0,
            BearerCapacityKind::Futexes => 0,
        }
    }
}

fn main() {
    // Bring up a listener
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let listener_port = match listener.local_addr().unwrap() {
        SocketAddr::V4(addr) => addr.port(),
        SocketAddr::V6(_) => panic!("IPv4 Requested, IPv6 bound?"),
    };

    println!("std TcpListener on 127.0.0.1:{listener_port}");

    // Now submit a completion for Connect OpCode against the std listener
    let my_cap = Capacity::<MyCapacity, BearerCapacityKind>::with_planned(MyCapacity {});
    let mut bearer = UringBearer::with_capacity(my_cap).unwrap();

    let sock = unsafe { libc::socket(libc::AF_INET, libc::SOCK_STREAM, libc::IPPROTO_TCP) };
    bearer
        .io_uring()
        .submitter()
        .register_files(&[sock])
        .unwrap();

    let ysaddr = YSockAddrR::from_sockaddr(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        listener_port,
    ));

    let _op_idx = bearer
        .push_connect(Connect::with_ysockaddr_c(0, ysaddr.as_c()).unwrap())
        .unwrap();

    bearer.submit_and_wait(1).unwrap();

    #[derive(Debug)]
    struct UserData {
        e: u32,
    }

    let mut user = UserData { e: 0 };
    let mut wait_count = 0;

    loop {
        bearer
            .completions(&mut user, |user, entry, rec| match rec {
                Completion::Connect(c) => {
                    user.e += 1;
                    println!(
                        "Connected Q<{:?}> Fixed_fd<{}> ysaddr<{:?}",
                        entry,
                        c.fixed_fd(),
                        c.ysaddr()
                    );
                    // no error
                    assert_eq!(entry.result(), 0);
                }
                _ => panic!("Queue had something else than Connect?"),
            })
            .unwrap();

        if user.e != 0 {
            break;
        }

        if wait_count > 4 {
            panic!("wait_count > 5 on Connect example.");
        }

        wait_count += 1;
        println!("Waiting for the completion @ {wait_count} ..");
        let st = std::time::Duration::from_secs(1);
        std::thread::sleep(st);
    }

    // Now check that the listener got the connection.
    match listener.accept() {
        Ok((in_s, in_a)) => println!(
            "std TcpListener accepted {} from {}",
            in_s.as_raw_fd(),
            in_a
        ),
        Err(e) => panic!("std TcpListener resulted in error = {e}"),
    }
}
