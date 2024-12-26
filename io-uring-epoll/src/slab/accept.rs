//! Accept builder

use crate::RawFd;
use core::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

#[derive(Clone, Debug, PartialEq)]
pub enum AcceptRec {
    V4(Accept4),
    V6(Accept6),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Accept4 {
    pub(crate) sockaddr: libc::sockaddr_in,
    pub(crate) socklen_t: libc::socklen_t,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Accept6 {
    pub(crate) sockaddr: libc::sockaddr_in6,
    pub(crate) socklen_t: libc::socklen_t,
}

impl AcceptRec {
    pub fn sockaddr(&self) -> Option<SocketAddr> {
        println!("Self = {:?}", self);

        let socklen_t = match self {
            AcceptRec::V4(r) => r.socklen_t,
            AcceptRec::V6(r) => r.socklen_t,
        };

        if socklen_t <= 0 {
            return None;
        }

        let family = match self {
            AcceptRec::V4(r) => r.sockaddr.sin_family,
            AcceptRec::V6(r) => r.sockaddr.sin6_family,
        };

        if i32::from(family) == libc::AF_INET6 && socklen_t != size_of::<libc::sockaddr_in>() as u32
        {
            return None;
        }

        if i32::from(family) == libc::AF_INET6
            && socklen_t != size_of::<libc::sockaddr_in6>() as u32
        {
            return None;
        }

        let port_be = match self {
            AcceptRec::V4(r) => r.sockaddr.sin_port,
            AcceptRec::V6(r) => r.sockaddr.sin6_port,
        };

        let accept_port = u16::from_be(port_be);

        let accept_ip = match self {
            AcceptRec::V4(r) => IpAddr::V4(Ipv4Addr::from_bits(u32::from_be(
                r.sockaddr.sin_addr.s_addr,
            ))),
            AcceptRec::V6(r) => {
                let in_bits = u128::from_be_bytes(r.sockaddr.sin6_addr.s6_addr);

                IpAddr::V6(Ipv6Addr::from_bits(in_bits))
            }
        };
        Some(SocketAddr::new(accept_ip, accept_port))
    }
}

#[inline]
pub(super) fn init_accept_rec4() -> AcceptRec {
    AcceptRec::V4(Accept4 {
        sockaddr: unsafe { std::mem::zeroed() },
        socklen_t: size_of::<libc::sockaddr>() as u32,
    })
}

#[inline]
pub(super) fn init_accept_rec6() -> AcceptRec {
    AcceptRec::V6(Accept6 {
        sockaddr: unsafe { std::mem::zeroed() },
        socklen_t: size_of::<libc::sockaddr_in6>() as u32,
    })
}

use io_uring::types::DestinationSlot;
use std::ptr::addr_of;

#[inline]
pub(super) fn entry(
    fd: RawFd,
    rec_w: &AcceptRec,
    slot: Option<DestinationSlot>,
    flags: i32,
) -> io_uring::squeue::Entry {
    let (mut addr_ptr, socklen_t_ptr) = match rec_w {
        AcceptRec::V4(rec) => {
            let mut addr_ptr = std::ptr::addr_of!(rec.sockaddr);
            let mut socklen_t_ptr = std::ptr::addr_of!(rec.socklen_t);
            (addr_ptr as *mut libc::sockaddr, socklen_t_ptr)
        }
        AcceptRec::V6(rec) => {
            let mut addr_ptr = std::ptr::addr_of!(rec.sockaddr);
            let mut socklen_t_ptr = std::ptr::addr_of!(rec.socklen_t);
            (addr_ptr as *mut libc::sockaddr, socklen_t_ptr)
        }
    };

    io_uring::opcode::Accept::new(
        io_uring::types::Fixed(fd as u32),
        addr_ptr,
        socklen_t_ptr as *mut libc::socklen_t,
    )
    .file_index(slot)
    .flags(flags)
    .build()
}
