//! magic

use core::pin::Pin;
use io_uring_opcode::{OpCompletion, OpCode, OpError};

#[cfg(feature = "connect")]
pub use io_uring_op_connect::Connect;

#[cfg(feature = "socket")]
pub use io_uring_op_socket::Socket;

/// unreachable
#[derive(Clone, Debug)]
pub enum WrapperError {
}

/// Wrapper for all the possible OpCodes
#[derive(Clone, Debug)]
pub enum Wrapper {
    /// Connect OpCode
    #[cfg(feature = "connect")]
    Connect(Connect),
    /// Socket OpCode
    #[cfg(feature = "socket")]
    Socket(Socket),
}

impl OpCompletion for Wrapper {
    type Error = WrapperError;
    #[inline]
    fn entry(&self) -> io_uring::squeue::Entry {
        match self {
            Self::Connect(i) => i.entry(),
            Self::Socket(i) => i.entry(),
        }        
    }
    #[inline]
    fn owner(&self) -> io_uring_owner::Owner {
        match self {
            Self::Connect(i) => i.owner(),
            Self::Socket(i) => i.owner(),
        }
    }
    #[inline]
    fn force_owner_kernel(&mut self) -> bool {
        match self {        
            Self::Connect(i) => i.force_owner_kernel(),
            Self::Socket(i) => i.force_owner_kernel(),
        }
    }
}

//pub trait OpExtConnect {
//    /// Underlying Fixed Fd ref
//    fn fixed_fd(&self) -> u32;
//    /// Underlying YSockAddr
//    fn ysaddr(&self) -> &YSockAddrC;
//}

#[cfg(feature = "connect")]
impl OpCode<Connect> for Wrapper {
    fn submission(self) -> Result<Connect, OpError> { todo!() }
    fn completion(&mut self, _: Pin<&mut Connect>) -> Result<(), OpError> { todo!() }
}

#[cfg(feature = "socket")]
impl OpCode<Socket> for Wrapper {
    fn submission(self) -> Result<Socket, OpError> { todo!() }
    fn completion(&mut self, _: Pin<&mut Socket>) -> Result<(), OpError> { todo!() }
}

impl OpCode<Wrapper> for Wrapper {
    fn submission(self) -> Result<Wrapper, OpError> { todo!() }
    fn completion(&mut self, _: Pin<&mut Wrapper>) -> Result<(), OpError> { todo!() }
}

impl OpCode<Wrapper> for Socket {
    #[inline]
    fn submission(self) -> Result<Wrapper, OpError> { Ok(Wrapper::Socket(self)) }
    fn completion(&mut self, _: Pin<&mut Wrapper>) -> Result<(), OpError> { todo!() }
}

impl OpCode<Wrapper> for Connect {
    #[inline]
    fn submission(self) -> Result<Wrapper, OpError> { Ok(Wrapper::Connect(self)) }
    fn completion(&mut self, _: Pin<&mut Wrapper>) -> Result<(), OpError> { todo!() }
}


impl Wrapper {
    /// Unwrap Socket type
    #[inline]
    pub fn unwrap_socket(&self) -> &Socket {
        match self {
            Self::Socket(ref s) => s,
            _ => panic!("Invalid Unwrap - Not a Socket"),
        }
    }
    /// Unwrap Connect type
    #[inline]
    pub fn unwrap_connect(&self) -> &Connect {
        match self {
            Self::Connect(ref c) => c,
            _ => panic!("Invalid Unwrap - Not a Connect"),
        }
    }
    /// unreachable
    #[inline]
    pub fn fixed_fd(&self) -> u32 {
        todo!()
    }
}
