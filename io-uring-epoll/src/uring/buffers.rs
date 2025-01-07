//! ProvideBuffers OpCode API Surface

impl UringHandler {
    /// Provide dynamically allocaed buffers, place these into slab and register.
    pub fn provide_buffers(bgid: u16, nbufs: u16, len: i32) -> Result<(), UringHandlerError> {
        let entry = self.buf_register.vacant_entry();
        let key = entry.key();

    }
    /// Register buffers through a raw pointer.
    ///
    /// # Safety
    ///
    /// User must ensure the buffers are valid until it is confirmed as de-registered
    /// and that the buffer must be initialized to zero to the valid len.
    pub unsafe fn raw_provide_buffers(buf: *mut u8, len: i32, nbufs: u16, bgid: u16, bid: u16) -> Result<usize, UringHandlerError> {
    let mut buf_in: [u8; 16384] = unsafe { std::mem::zeroed() };
        let p_buffers_rec = io_uring::opcode::ProvideBuffers::new(
            buf, len, nbufs, bgid, bid
        ).build();
        let s = unsafe { s_queue.push(&p_buffers_rec) };

        
    }
    
}
