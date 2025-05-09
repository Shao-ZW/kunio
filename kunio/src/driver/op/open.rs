use std::ffi::CString;
use std::io;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use super::Op;
use super::UringOp;

use io_uring::{opcode, types};

pub struct Open {
    path: CString,
    flags: i32,
    mode: libc::mode_t,
}

impl UringOp for Open {
    fn build_sqe(&mut self) -> io_uring::squeue::Entry {
        opcode::OpenAt::new(types::Fd(libc::AT_FDCWD), self.path.as_c_str().as_ptr())
            .flags(self.flags)
            .mode(self.mode)
            .build()
    }
}

impl Op<Open> {
    pub fn open<P: AsRef<Path>>(path: P, flags: i32, mode: libc::mode_t) -> io::Result<Op<Open>> {
        let path = CString::new(path.as_ref().as_os_str().as_bytes())?;
        crate::runtime::RUNTIME.with(|runtime| runtime.driver.submit_op(Open { path, flags, mode }))
    }
}
