use std::io;
use std::os::fd::RawFd;

use super::Op;
use super::UringOp;

use io_uring::{opcode, types};

use crate::buf::IoBufMut;

pub struct Recv<T> {
    fd: RawFd,
    pub buf: T,
}

impl<T: IoBufMut> UringOp for Recv<T> {
    fn build_sqe(&mut self) -> io_uring::squeue::Entry {
        opcode::Recv::new(
            types::Fd(self.fd),
            self.buf.write_ptr(),
            self.buf.available_len(),
        )
        .build()
    }
}

impl<T: IoBufMut> Op<Recv<T>> {
    pub fn recv(fd: RawFd, buf: T) -> io::Result<Op<Recv<T>>> {
        crate::runtime::RUNTIME.with(|runtime| runtime.driver.submit_op(Recv { fd, buf }))
    }
}
