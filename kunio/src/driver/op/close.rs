use std::io;
use std::os::fd::RawFd;

use super::Op;
use super::UringOp;

use crate::runtime::RUNTIME;
use io_uring::{opcode, types};

pub struct Close {
    fd: types::Fd,
}

impl UringOp for Close {
    fn build_sqe(&mut self) -> io_uring::squeue::Entry {
        opcode::Close::new(self.fd).build()
    }
}

impl Op<Close> {
    pub fn close(fd: RawFd) -> io::Result<Op<Close>> {
        RUNTIME.with(|runtime| runtime.driver.submit_op(Close { fd: types::Fd(fd) }))
    }
}
