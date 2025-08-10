use io_uring_bearer::Completion;
use io_uring_bearer::UringBearer;

use capacity::{Capacity, Setting};
use io_uring_bearer::BearerCapacityKind;

use io_uring_op_socket::Socket;
use io_uring_op_socket::TargetFdType;
use io_uring_opcode::OpExtSocket;

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
    let my_cap = Capacity::<MyCapacity, BearerCapacityKind>::with_planned(MyCapacity {});
    let mut bearer = UringBearer::with_capacity(my_cap).unwrap();

    let _op_idx = bearer
        .push_socket(
            Socket::with_regular_fd(libc::AF_INET, libc::SOCK_STREAM, libc::IPPROTO_TCP).unwrap(),
        )
        .unwrap();

    bearer.submit().unwrap();

    #[derive(Debug)]
    struct UserData {
        e: u32,
    }

    let mut user = UserData { e: 0 };
    let mut wait_count = 0;

    loop {
        bearer
            .completions(&mut user, |user, entry, rec| match rec {
                Completion::Socket(c) => {
                    user.e += 1;
                    println!("Socketed Q<{:?}> ", entry,);
                    // no error - return here is kernel assigned filehandle number
                    assert!(entry.result() > 0);
                    assert_eq!(c.target_fd(), TargetFdType::Regular);
                    assert_eq!(c.domain(), libc::AF_INET);
                    assert_eq!(c.socket_type(), libc::SOCK_STREAM);
                    assert_eq!(c.protocol(), libc::IPPROTO_TCP);
                }
                _ => panic!("Queue had something else than Socket?"),
            })
            .unwrap();

        if user.e != 0 {
            break;
        }

        if wait_count > 4 {
            panic!("wait_count > 5 on Socket example.");
        }

        wait_count += 1;
        println!("Waiting for the completion @ {wait_count} ..");
        let st = std::time::Duration::from_millis(50);
        std::thread::sleep(st);
    }
}
