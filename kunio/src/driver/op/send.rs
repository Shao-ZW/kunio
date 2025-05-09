use std::io;
use std::os::fd::RawFd;

use super::Op;
use super::UringOp;

use io_uring::{opcode, types};

use crate::buf::IoBuf;

pub struct Send<T> {
    fd: RawFd,
    pub buf: T,
}

impl<T: IoBuf> UringOp for Send<T> {
    fn build_sqe(&mut self) -> io_uring::squeue::Entry {
        opcode::Send::new(
            types::Fd(self.fd),
            self.buf.read_ptr(),
            self.buf.valid_len(),
        )
        .build()
    }
}

impl<T: IoBuf> Op<Send<T>> {
    pub fn send(fd: RawFd, buf: T) -> io::Result<Op<Send<T>>> {
        crate::runtime::RUNTIME.with(|runtime| runtime.driver.submit_op(Send { fd, buf }))
    }
}
