use std::io;
use std::mem::MaybeUninit;
use std::os::fd::RawFd;

use super::Op;
use super::UringOp;

use io_uring::{opcode, types};

pub struct Accept {
    fd: RawFd,
    pub addr: Box<(
        MaybeUninit<libc::sockaddr_storage>,
        MaybeUninit<libc::socklen_t>,
    )>,
}

impl UringOp for Accept {
    fn build_sqe(&mut self) -> io_uring::squeue::Entry {
        opcode::Accept::new(
            types::Fd(self.fd),
            self.addr.0.as_mut_ptr() as *mut _,
            self.addr.1.as_mut_ptr() as *mut _,
        )
        .build()
    }
}

impl Op<Accept> {
    pub fn accept(fd: RawFd) -> io::Result<Op<Accept>> {
        crate::runtime::RUNTIME.with(|runtime| {
            runtime.driver.submit_op(Accept {
                fd,
                addr: Box::new((MaybeUninit::uninit(), MaybeUninit::uninit())),
            })
        })
    }
}
