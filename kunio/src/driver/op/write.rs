use std::io;
use std::os::fd::RawFd;

use super::Op;
use super::UringOp;

use io_uring::{opcode, types};

use crate::buf::IoBuf;

pub struct Write<T> {
    fd: types::Fd,
    pub buf: T,
}

impl<T: IoBuf> UringOp for Write<T> {
    fn build_sqe(&mut self) -> io_uring::squeue::Entry {
        opcode::Write::new(self.fd, self.buf.read_ptr(), self.buf.size())
            .offset(-1i64 as u64)
            .build()
    }
}

impl<T: IoBuf> Op<Write<T>> {
    pub fn write(fd: RawFd, buf: T) -> io::Result<Op<Write<T>>> {
        crate::runtime::RUNTIME.with(|runtime| {
            runtime.driver.submit_op(Write {
                fd: types::Fd(fd),
                buf,
            })
        })
    }
}

pub struct WriteAt<T> {
    fd: types::Fd,
    pub buf: T,
    offset: u64,
}

impl<T: IoBuf> UringOp for WriteAt<T> {
    fn build_sqe(&mut self) -> io_uring::squeue::Entry {
        opcode::Write::new(self.fd, self.buf.read_ptr(), self.buf.size())
            .offset(self.offset)
            .build()
    }
}

impl<T: IoBuf> Op<WriteAt<T>> {
    pub fn write_at(fd: RawFd, buf: T, offset: u64) -> io::Result<Op<WriteAt<T>>> {
        crate::runtime::RUNTIME.with(|runtime| {
            runtime.driver.submit_op(WriteAt {
                fd: types::Fd(fd),
                buf,
                offset,
            })
        })
    }
}
