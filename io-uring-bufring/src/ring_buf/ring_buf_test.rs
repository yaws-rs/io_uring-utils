//! RingBuf Tests

use super::*;
use anonymous_mmap::AnonymousMmap;

#[test]
fn choice_default_pagesize_ok() {
    let buffer_count = BufferCount(unsafe { NonZero::new_unchecked(2) });
    let per_buffer_size = PerBufferSize(unsafe { NonZero::new_unchecked(8192) });
    let choice = RingBufChoice::with_default_pagesize(buffer_count, per_buffer_size).unwrap();
    assert_eq!(choice.total_bufs_count(), 2);
    assert_eq!(choice.total_bufs_size(), 16384);
    assert_eq!(choice.per_bufsize(), 8192);
}

#[test]
#[should_panic]
fn choice_default_pagesize_err() {
    let buffer_count = BufferCount(unsafe { NonZero::new_unchecked(2) });
    let per_buffer_size = PerBufferSize(unsafe { NonZero::new_unchecked(8191) });
    RingBufChoice::with_default_pagesize(buffer_count, per_buffer_size).unwrap();
}

fn _create_unreg() -> Result<(AnonymousMmap, RingBufUnregistered), RingBufError> {
    let buffer_count = BufferCount(unsafe { NonZero::new_unchecked(2) });
    let per_buffer_size = PerBufferSize(unsafe { NonZero::new_unchecked(8192) });
    let choice = RingBufChoice::with_default_pagesize(buffer_count, per_buffer_size).unwrap();

    let zeroed_buf = AnonymousMmap::new(16384).unwrap();
    let zeroed_buf_ptr = zeroed_buf.as_ptr_mut() as *mut u8;
    let unreg = unsafe { RingBufUnregistered::with_rawbuf_continuous(choice, zeroed_buf_ptr) }?;

    Ok((zeroed_buf, unreg))
}

#[test]
fn create_unreg_default_pagesize_ok() {
    _create_unreg().unwrap();
}

#[cfg(feature = "bearer")]
mod bearer_test {

    use super::*;
    use capacity::{Capacity, Setting};
    use io_uring_bearer::{UringBearer, error::UringBearerError, BearerCapacityKind, TargetFd, completion::SubmissionRecordStatus, Completion};
    use io_uring_opcode_sets::{Socket, Connect, Wrapper};
    use io_uring_bearer::SubmissionFlags;
    use ysockaddr::YSockAddrR;
    
    #[derive(Clone, Debug)]
    struct TestCapacity;
    
    impl Setting<BearerCapacityKind> for TestCapacity {
        fn setting(&self, _v: &BearerCapacityKind) -> usize {
            16
        }
    }

    fn _create_bearer() -> Result<UringBearer<Wrapper>, UringBearerError> {
        let cap = Capacity::<TestCapacity, BearerCapacityKind>::with_planned(TestCapacity);
        UringBearer::<Wrapper>::with_capacity(cap)
    }
    
    #[test]
    fn create_reg_default_pagesize_ok() {
        let mut bearer = _create_bearer().unwrap();
        let (_raw_buf, unreg) = _create_unreg().unwrap();

        unreg.register_with_bearer(&mut bearer, 666).unwrap();

        // _raw_buf leaks memory as Drop is fallible needing manual drop
    }

    #[test]
    fn play_reg_default_pagesize_ok() {
        let (raw_buf_client_in, ringbuf_client_in_unreg) = _create_unreg().unwrap();
        let (raw_buf_client_out, ringbuf_client_out_unreg) = _create_unreg().unwrap();
        let (raw_buf_server_in, ringbuf_server_in_unreg) = _create_unreg().unwrap();
        let (raw_buf_server_out, ringbuf_server_out_unreg) = _create_unreg().unwrap();        

        let mut client_bearer = _create_bearer().unwrap();
        let mut server_bearer = _create_bearer().unwrap();

        use std::os::fd::AsRawFd;
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let local_addr = listener.local_addr().unwrap();
        let listener_raw_fd = listener.as_raw_fd();
        let listener_ysaddr = YSockAddrR::from_sockaddr(local_addr);
        
        let ringbuf_client_in =
            ringbuf_client_in_unreg.register_with_bearer(&mut client_bearer, 100).unwrap();
        
        let ringbuf_client_out =
            ringbuf_client_out_unreg.register_with_bearer(&mut client_bearer, 150).unwrap();

        let ringbuf_server_in =
            ringbuf_server_in_unreg.register_with_bearer(&mut server_bearer, 200).unwrap();
        let ringbuf_server_out =
            ringbuf_server_out_unreg.register_with_bearer(&mut server_bearer, 250).unwrap();        

        let flags_socket = Some(SubmissionFlags::default().on_io_link());
        let flags_connect: Option<SubmissionFlags> = None;

        // intialize the client fd mapping
        client_bearer
            .io_uring()
            .submitter()
            .register_files(&[-1])
            .unwrap();

        // intialize the server fd mapping
        server_bearer
            .io_uring()
            .submitter()
            .register_files(&[listener_raw_fd, -1])
            .unwrap();

        // push hard-linked socket-connect for the client
        client_bearer
            .push_socket(
                Socket::with_fixed_fd(
                    Some(0_u32),
                    libc::AF_INET,
                    libc::SOCK_STREAM,
                    libc::IPPROTO_TCP,
                ).unwrap(),
                flags_socket,
            ).unwrap();
        client_bearer
            .push_connect(
                Connect::with_ysockaddr_c(0_u32, listener_ysaddr.as_c()).unwrap(),
                flags_connect,
            )
            .unwrap();

        unsafe { server_bearer.add_accept_ipv4(0, TargetFd::ManualRegistered(1)) }.unwrap();
        
        client_bearer.submit();
        server_bearer.submit();

        #[derive(Default)]
        struct ServerData {
            ready: bool,
            recv_sent: bool,
            send_sent: bool,
            client_addr: Option<core::net::SocketAddr>,
            got_ping: bool,            
        }

        #[derive(Default)]
        struct ClientData {
            ready: bool,
            recv_sent: bool,
            send_sent: bool,
            got_pong: bool,
            
        }
        let mut server_data = ServerData::default();
        let mut client_data = ClientData::default();
        
        loop {

            if client_data.got_pong {
                break;
            }
            
            if client_data.ready && !client_data.recv_sent {
                client_bearer.add_recv_multi(0, 100, None).unwrap();
                client_data.recv_sent = true;
                client_bearer.submit();
            }

            if client_data.ready && !client_data.send_sent {
                let surf = unsafe { core::slice::from_raw_parts_mut(raw_buf_client_out.as_ptr_mut() as *mut u8, 4) };
                surf.copy_from_slice("PING".as_bytes());
                unsafe { client_bearer.add_send_zc_rawbuf(0, raw_buf_client_out.as_ptr_mut() as _, 4_u32, None, None); }
                client_data.send_sent = true;
                client_bearer.submit();
            }
            
            if server_data.ready && !server_data.recv_sent {
                server_bearer.add_recv_multi(1, 200, None).unwrap();
                server_data.recv_sent = true;
                server_bearer.submit();
            }

            if server_data.ready && server_data.got_ping && !server_data.send_sent{
                let surf = unsafe { core::slice::from_raw_parts_mut(raw_buf_server_out.as_ptr_mut() as *mut u8, 4) };
                surf.copy_from_slice("PONG".as_bytes());
                unsafe { server_bearer.add_send_zc_rawbuf(1, raw_buf_server_out.as_ptr_mut() as _, 4_u32, None, None); }
                server_data.send_sent = true;
                server_bearer.submit();
            }
            
            unsafe { client_bearer.handle_completions(&mut client_data, None, |cdata, entry, rec| {
                match rec {
                    Completion::Socket(s) => {
                        assert_eq!(entry.result(), 0);
                        SubmissionRecordStatus::Forget
                    },
                    Completion::Connect(c) => {
                        assert_eq!(entry.result(), 0);
                        cdata.ready = true;
                        SubmissionRecordStatus::Forget
                    },
                    Completion::RecvMulti(r) => {
                        assert_eq!(entry.result(), 4);
                        cdata.got_pong = true;
                        SubmissionRecordStatus::Forget
                    },
                    Completion::SendZc(s) => {
                        assert_eq!(entry.result(), 4);
                        SubmissionRecordStatus::Forget
                    },
                    _ => panic!("client_bearer Unhandled entry = {:?}, rec = {:?}", entry, rec),
                }
            }) };
            unsafe { server_bearer.handle_completions(&mut server_data, None, |sdata, entry, rec| {
                match rec {
                    Completion::Accept(a) => {
                        assert_eq!(entry.result(), 0);

                        sdata.ready = true;
                        sdata.client_addr = a.sockaddr();
                        SubmissionRecordStatus::Forget
                    },
                    Completion::SendZc(s) => {
                        assert_eq!(entry.result(), 4);
                        SubmissionRecordStatus::Forget
                    },
                    Completion::RecvMulti(r) => {
                        assert_eq!(entry.result(), 4);
                        sdata.got_ping = true;
                        SubmissionRecordStatus::Forget
                    },
                    _ => panic!("server_bearer Unhandled entry = {:?}, rec = {:?}", entry, rec),                    
                }
            }) };
        }
    }
    
}
