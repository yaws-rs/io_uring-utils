use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::os::fd::AsRawFd;

use io_uring_bearer::Completion;
use io_uring_bearer::UringBearer;

use capacity::{Capacity, Setting};
use io_uring_bearer::BearerCapacityKind;

use io_uring_epoll::{EpollCtl, EpollUringHandler, HandledFd};

#[derive(Clone, Debug)]
pub struct MyCapacity;

impl Setting<BearerCapacityKind> for MyCapacity {
    fn setting(&self, v: &BearerCapacityKind) -> usize {
        match v {
            BearerCapacityKind::CoreQueue => 16,
            BearerCapacityKind::RegisteredFd => 16,
            BearerCapacityKind::PendingCompletions => 16,
            BearerCapacityKind::Buffers => 16,
            BearerCapacityKind::Futexes => 16,
        }
    }
}

fn main() {
    let my_cap = Capacity::<MyCapacity, BearerCapacityKind>::with_planned(MyCapacity {});
    let mut bearer = UringBearer::with_capacity(my_cap).unwrap();
    let ep_uring_handler = EpollUringHandler::with_bearer(&mut bearer).unwrap();
    let epfd = ep_uring_handler.epfd();

    // This works with any impl that provides std::os::fd::AsRawFd impl
    // In POSIX/UNIX-like it's just i32 file number or "fileno"
    // The user is responsible of keeping the filehandle "alive" and should use OwnedFd as base
    let listen =
        std::net::TcpListener::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0))
            .unwrap();

    // Add the listen handle into EpollHandler
    let mut handle_fd = HandledFd::from_raw(listen.as_raw_fd());
    let set_mask = handle_fd.set_in(true);
    assert_eq!(set_mask, 1);

    // Create EpollCtl Op
    let epoll_ctl = EpollCtl::with_epfd_handled(epfd, handle_fd, 666).unwrap();

    // Push the EpollCtl into the UringBearer
    let ctl_idx = bearer.push_op(epoll_ctl).unwrap();

    // This is the indexed EpollCtl index for later modifications.
    println!("EpollCtl Index is = {}", ctl_idx);

    // Bearer to commit the pushed EpollCtl and wait it's sole completion.
    bearer.submit_and_wait(1).unwrap();

    #[derive(Debug)]
    struct UserData {
        e: u32,
    }

    let mut user = UserData { e: 0 };

    // This is unsafe because forgetting the original submission record
    // can lead to UB where the kernel still refers to it's address later.
    /*
    unsafe { bearer.handle_completions(&mut user, |user, entry, rec| {
        match rec {
            Completion::EpollEvent(e) => {
                user.e += 1;
                println!("EpollEvent => {:?} Entry => {:?}", e, entry);
                // SAFETY: We retain the record and will not move it
                SubmissionRecordStatus::Retain
            }
            _ => todo!(),
        }
    }) };
    */

    // This is safe since we are not touching the original submission record.
    bearer
        .completions(&mut user, |user, entry, rec| match rec {
            Completion::Op(e) => {
                user.e += 1;
                println!("Completed => {:?} Entry => {:?}", e, entry);
            }
            _ => todo!(),
        })
        .unwrap();
}
