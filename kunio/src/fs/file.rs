use std::io;
use std::os::fd::RawFd;
use std::path::Path;

use crate::buf::{IoBuf, IoBufMut};
use crate::driver::op::Op;

pub struct File {
    fd: RawFd,
}

impl File {
    pub async fn create<P: AsRef<Path>>(path: P) -> io::Result<File> {
        let op = Op::open(path, libc::O_CREAT | libc::O_RDWR, 0o644)?;
        let completion = op.await;
        Ok(File {
            fd: completion.result?,
        })
    }

    pub async fn open<P: AsRef<Path>>(path: P) -> io::Result<File> {
        let op = Op::open(path, libc::O_RDWR, 0o644)?;
        let completion = op.await;
        Ok(File {
            fd: completion.result?,
        })
    }

    pub async fn close(&self) -> io::Result<()> {
        Op::close(self.fd)?.await;
        Ok(())
    }

    pub async fn write<T: IoBuf>(&self, buf: T) -> io::Result<(usize, T)> {
        let op = Op::write(self.fd, buf)?;
        let completion = op.await;
        Ok((completion.result? as usize, completion.data.buf))
    }

    pub async fn read<T: IoBufMut>(&self, buf: T) -> io::Result<(usize, T)> {
        let op = Op::read(self.fd, buf)?;
        let mut completion = op.await;
        let result = completion.result?;
        // Safety:
        unsafe {
            completion.data.buf.set_valid_len(result as u32);
        }
        Ok((result as usize, completion.data.buf))
    }

    pub async fn write_at<T: IoBuf>(&self, buf: T, pos: u64) -> io::Result<(usize, T)> {
        let op = Op::write_at(self.fd, buf, pos)?;
        let completion = op.await;
        Ok((completion.result? as usize, completion.data.buf))
    }

    pub async fn read_at<T: IoBufMut>(&self, buf: T, pos: u64) -> io::Result<(usize, T)> {
        let op = Op::read_at(self.fd, buf, pos)?;
        let mut completion = op.await;
        let result = completion.result?;
        unsafe {
            completion.data.buf.set_valid_len(result as u32);
        }
        Ok((result as usize, completion.data.buf))
    }
}
