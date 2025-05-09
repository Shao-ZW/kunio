use std::io;

use super::Op;
use super::UringOp;

use io_uring::opcode;

// Important !!! linux kernel 5.19 least
pub struct Socket {
    domain: i32,
    socket_type: i32,
    protocol: i32,
}

impl UringOp for Socket {
    fn build_sqe(&mut self) -> io_uring::squeue::Entry {
        opcode::Socket::new(self.domain, self.socket_type, self.protocol).build()
    }
}

impl Op<Socket> {
    pub fn socket(domain: i32, socket_type: i32, protocol: i32) -> io::Result<Op<Socket>> {
        crate::runtime::RUNTIME.with(|runtime| {
            runtime.driver.submit_op(Socket {
                domain,
                socket_type,
                protocol,
            })
        })
    }
}
