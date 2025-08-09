//! Socket Record

use core::pin::Pin;

use crate::error::SocketError;

use io_uring_opcode::types::TargetFdType;
use io_uring_opcode::OpExtSocket;
use io_uring_opcode::{OpCode, OpCompletion, OpError};
use io_uring_owner::Owner;

/// Socket Record
#[derive(Clone, Debug)]
pub struct Socket {
    // Current owner of the record
    pub(crate) owner: Owner,
    // Socket domain, e.g. AF_INET
    pub(crate) domain: i32,
    // Socket type, e.g. SOCK_STREAM
    pub(crate) socket_type: i32,
    // Socket protocol, e.g IPPROTO_TCP
    pub(crate) protocol: i32,
    // Underlying targeted Fd type and it's slot index if any.
    pub(crate) target_fd_type: TargetFdType,
    // impl detail - this is one-way thus we have to keep both fixed_fd and this :(
    pub(crate) target_destination_slot: Option<io_uring::types::DestinationSlot>,
}

impl Socket {
    /// Construct a new regular Socket
    pub fn with_regular_fd(
        domain: i32,
        socket_type: i32,
        protocol: i32,
    ) -> Result<Self, SocketError> {
        Ok(Socket {
            owner: Owner::Created,
            domain,
            socket_type,
            protocol,
            target_fd_type: TargetFdType::Regular,
            target_destination_slot: None,
        })
    }
    /// Construct a new fixed Socket, optionally targeting specific fixed slot or auto by default.
    /// On auto there must be free (as in set -1) registered entries available.
    pub fn with_fixed_fd(
        manual_target_slot: Option<u32>,
        domain: i32,
        socket_type: i32,
        protocol: i32,
    ) -> Result<Self, SocketError> {
        // This is one-way construct
        let target_destination_slot = match manual_target_slot {
            None => io_uring::types::DestinationSlot::auto_target(),
            Some(s) => match io_uring::types::DestinationSlot::try_from_slot_target(s) {
                Err(_) => return Err(SocketError::InvalidTarget(s)),
                Ok(dest_slot) => dest_slot,
            },
        };

        // ... so we have to take a copy for later feedback
        let target_fd_type = match manual_target_slot {
            None => TargetFdType::FixedAuto,
            Some(s) => TargetFdType::FixedManual(s),
        };

        Ok(Socket {
            owner: Owner::Created,
            domain,
            socket_type,
            protocol,
            target_fd_type,
            target_destination_slot: Some(target_destination_slot),
        })
    }
}

impl OpCompletion for Socket {
    type Error = OpError;
    fn entry(&self) -> io_uring::squeue::Entry {
        io_uring::opcode::Socket::new(self.domain, self.socket_type, self.protocol)
            .file_index(self.target_destination_slot)
            .build()
    }
    fn owner(&self) -> Owner {
        self.owner.clone()
    }
    fn force_owner_kernel(&mut self) -> bool {
        self.owner = Owner::Kernel;
        true
    }
}

impl OpCode<Socket> for Socket {
    fn submission(self) -> Result<Socket, OpError> {
        Ok(self)
    }
    fn completion(&mut self, _: Pin<&mut Socket>) -> Result<(), OpError> {
        todo!()
    }
}

impl OpExtSocket for Socket {
    /// Underlying domain
    fn domain(&self) -> i32 {
        self.domain
    }
    /// Underlying socket type
    fn socket_type(&self) -> i32 {
        self.socket_type
    }
    /// Underlying protocol
    fn protocol(&self) -> i32 {
        self.protocol
    }
    /// Underlying targeted Fd type and slot if any
    fn target_fd(&self) -> TargetFdType {
        self.target_fd_type.clone()
    }
}
