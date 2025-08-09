use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::net::{TcpListener, TcpStream};
use std::os::fd::AsRawFd;
use std::time::Duration;

use io_uring_bearer::Completion;
use io_uring_bearer::UringBearer;

use capacity::{Capacity, Setting};
use io_uring_bearer::BearerCapacityKind;

use io_uring_op_accept_multi::AcceptMulti;
use io_uring_opcode::OpExtAcceptMulti;

#[derive(Clone, Debug)]
pub struct MyCapacity;

impl Setting<BearerCapacityKind> for MyCapacity {
    fn setting(&self, v: &BearerCapacityKind) -> usize {
        match v {
            BearerCapacityKind::CoreQueue => 10,
            BearerCapacityKind::RegisteredFd => 20,
            BearerCapacityKind::PendingCompletions => 20,
            BearerCapacityKind::Buffers => 0,
            BearerCapacityKind::Futexes => 0,
        }
    }
}

fn main() {
    // Bring up a listener
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    //listener.set_nonblocking(true).unwrap();
    let listener_port = match listener.local_addr().unwrap() {
        SocketAddr::V4(addr) => addr.port(),
        SocketAddr::V6(_) => panic!("IPv4 Requested, IPv6 bound?"),
    };

    println!("std TcpListener on 127.0.0.1:{listener_port}");

    // Now submit a completion for AcceptMulti OpCode against the std listener
    let my_cap = Capacity::<MyCapacity, BearerCapacityKind>::with_planned(MyCapacity {});
    let mut bearer = UringBearer::with_capacity(my_cap).unwrap();

    let listener_fd = listener.as_raw_fd();

    // Make room for two fixed Fds
    bearer
        .io_uring()
        .submitter()
        .register_files(&[listener_fd, -1, -1])
        .unwrap();

    let _op_idx = bearer
        .push_accept_multi(AcceptMulti::with_fixed_fds(0).unwrap())
        .unwrap();

    bearer.submit().unwrap();

    #[derive(Debug)]
    struct UserData {
        e: u32,
    }

    let mut user = UserData { e: 0 };
    let mut wait_count = 0;

    let conn_tm = Duration::new(1, 0);
    let conn_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), listener_port);
    let _client1 = TcpStream::connect_timeout(&conn_addr, conn_tm).unwrap();
    let _client2 = TcpStream::connect_timeout(&conn_addr, conn_tm).unwrap();

    loop {
        bearer
            .completions(&mut user, |user, entry, rec| match rec {
                Completion::AcceptMulti(c) => {
                    user.e += 1;
                    println!(
                        "Accepted Q<{:?}> Acceptor_fixed_fd<{}>",
                        entry,
                        c.fixed_fd(),
                    );
                    // multi-shot for accept should indicate there may be more later
                    assert_eq!(io_uring::cqueue::more(entry.flags()), true);
                    // no error - kernel dependant allocating the fixed fileno
                    assert!(entry.result() > 0);
                }
                _ => panic!("Queue had something else than AcceptMulti?"),
            })
            .unwrap();

        if user.e != 0 {
            break;
        }

        if wait_count > 4 {
            panic!("wait_count > 5 on AcceptMulti example.");
        }

        wait_count += 1;
        println!("Waiting for the completion @ {wait_count} ..");
        let st = std::time::Duration::from_secs(1);
        std::thread::sleep(st);
    }
}
