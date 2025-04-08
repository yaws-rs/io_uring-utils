use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::os::fd::AsRawFd;

use io_uring_bearer::Completion;
use io_uring_bearer::UringBearer;

use capacity::{Capacity, Setting};
use io_uring_bearer::BearerCapacityKind;

//use io_uring_epoll::{EpollCtl, EpollUringHandler, HandledFd};

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

    let op_idx = bearer.push_op(connect).unwrap();

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
