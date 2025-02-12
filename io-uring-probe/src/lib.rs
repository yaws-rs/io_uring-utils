//! io-uring probe

	#[allow(clippy::bool_comparison)]
	if epoll_probe.is_supported(EpollCtl::CODE) == false {
	    return Err(EpollUringHandlerError::NotSupported);
	}

	// SAFETY: FFI no-data in                                                                                                                                       
	let epfd = unsafe { libc::epoll_create1(0) };
        if epfd == -1 {
            // SAFETY: ffi no-data                                                                                                                                      
            let errno = unsafe { libc::__errno_location() };
            return Err(EpollUringHandlerError::EpollCreate1(format!(
                "errno: {:?}",
                errno
            )));
        }
