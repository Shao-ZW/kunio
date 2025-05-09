use std::io;
use std::os::fd::RawFd;

use super::Op;
use super::UringOp;

use io_uring::{opcode, types};

use crate::buf::IoBufMut;

pub struct Read<T> {
    fd: RawFd,
    pub buf: T,
}

impl<T: IoBufMut> UringOp for Read<T> {
    fn build_sqe(&mut self) -> io_uring::squeue::Entry {
        opcode::Read::new(
            types::Fd(self.fd),
            self.buf.write_ptr(),
            self.buf.available_len(),
        )
        .offset(-1i64 as u64)
        .build()
    }
}

impl<T: IoBufMut> Op<Read<T>> {
    pub fn read(fd: RawFd, buf: T) -> io::Result<Op<Read<T>>> {
        crate::runtime::RUNTIME.with(|runtime| runtime.driver.submit_op(Read { fd, buf }))
    }
}

pub struct ReadAt<T> {
    fd: RawFd,
    pub buf: T,
    offset: u64,
}

impl<T: IoBufMut> UringOp for ReadAt<T> {
    fn build_sqe(&mut self) -> io_uring::squeue::Entry {
        opcode::Read::new(
            types::Fd(self.fd),
            self.buf.write_ptr(),
            self.buf.available_len(),
        )
        .offset(self.offset)
        .build()
    }
}

impl<T: IoBufMut> Op<ReadAt<T>> {
    pub fn read_at(fd: RawFd, buf: T, offset: u64) -> io::Result<Op<ReadAt<T>>> {
        crate::runtime::RUNTIME.with(|runtime| runtime.driver.submit_op(ReadAt { fd, buf, offset }))
    }
}
