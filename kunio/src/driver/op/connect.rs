use std::io;
use std::net::SocketAddr;
use std::os::fd::RawFd;

use super::Op;
use super::UringOp;

use io_uring::{opcode, types};

pub struct Connect {
    fd: RawFd,
    addr: Box<SocketAddrCRepr>, // Boxed ! async reason
    addrlen: libc::socklen_t,
}

impl UringOp for Connect {
    fn build_sqe(&mut self) -> io_uring::squeue::Entry {
        opcode::Connect::new(types::Fd(self.fd), self.addr.as_ptr(), self.addrlen).build()
    }
}

impl Op<Connect> {
    pub fn connect(fd: RawFd, addr: SocketAddr) -> io::Result<Op<Connect>> {
        crate::runtime::RUNTIME.with(|runtime| {
            let (addr, addrlen) = socket_addr(&addr);
            runtime.driver.submit_op(Connect {
                fd,
                addr: Box::new(addr),
                addrlen,
            })
        })
    }
}

/// A type with the same memory layout as `libc::sockaddr`. Used in converting Rust level
/// SocketAddr* types into their system representation. The benefit of this specific
/// type over using `libc::sockaddr_storage` is that this type is exactly as large as it
/// needs to be and not a lot larger. And it can be initialized cleaner from Rust.
// Copied from mio.
#[repr(C)]
pub(crate) union SocketAddrCRepr {
    v4: libc::sockaddr_in,
    v6: libc::sockaddr_in6,
}

impl SocketAddrCRepr {
    pub(crate) fn as_ptr(&self) -> *const libc::sockaddr {
        self as *const _ as *const libc::sockaddr
    }
}

/// Converts a Rust `SocketAddr` into the system representation.
pub(crate) fn socket_addr(addr: &SocketAddr) -> (SocketAddrCRepr, libc::socklen_t) {
    match addr {
        SocketAddr::V4(addr) => {
            // `s_addr` is stored as BE on all machine and the array is in BE order.
            // So the native endian conversion method is used so that it's never swapped.
            let sin_addr = libc::in_addr {
                s_addr: u32::from_ne_bytes(addr.ip().octets()),
            };

            let sockaddr_in = libc::sockaddr_in {
                sin_family: libc::AF_INET as libc::sa_family_t,
                sin_port: addr.port().to_be(),
                sin_addr,
                sin_zero: [0; 8],
            };

            let sockaddr = SocketAddrCRepr { v4: sockaddr_in };
            let socklen = std::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t;
            (sockaddr, socklen)
        }
        SocketAddr::V6(addr) => {
            let sockaddr_in6 = libc::sockaddr_in6 {
                sin6_family: libc::AF_INET6 as libc::sa_family_t,
                sin6_port: addr.port().to_be(),
                sin6_addr: libc::in6_addr {
                    s6_addr: addr.ip().octets(),
                },
                sin6_flowinfo: addr.flowinfo(),
                sin6_scope_id: addr.scope_id(),
            };

            let sockaddr = SocketAddrCRepr { v6: sockaddr_in6 };
            let socklen = std::mem::size_of::<libc::sockaddr_in6>() as libc::socklen_t;
            (sockaddr, socklen)
        }
    }
}
