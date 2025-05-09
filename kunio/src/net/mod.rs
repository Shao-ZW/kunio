use std::os::fd::{AsRawFd, RawFd};
use std::{
    io,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6, ToSocketAddrs},
};

use crate::{
    buf::{IoBuf, IoBufMut},
    driver::op::Op,
};

pub struct TcpListener {
    listener: std::net::TcpListener,
}

pub struct TcpStream {
    fd: RawFd,
}

impl TcpListener {
    pub fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<TcpListener> {
        let listener = std::net::TcpListener::bind(addr)?;
        Ok(TcpListener { listener })
    }

    pub async fn accept(&self) -> io::Result<(TcpStream, SocketAddr)> {
        let op = Op::accept(self.listener.as_raw_fd())?;
        let completion = op.await;
        let stream = TcpStream {
            fd: completion.result?,
        };
        let storage = completion.data.addr.0.as_ptr();
        let addr = unsafe {
            match (*storage).ss_family as _ {
                libc::AF_INET => {
                    let addr = *(storage as *const libc::sockaddr_in);
                    SocketAddr::from(SocketAddrV4::new(
                        Ipv4Addr::from(addr.sin_addr.s_addr.to_ne_bytes()),
                        u16::from_be(addr.sin_port),
                    ))
                }
                libc::AF_INET6 => {
                    let addr = *(storage as *const libc::sockaddr_in6);
                    SocketAddr::from(SocketAddrV6::new(
                        Ipv6Addr::from(addr.sin6_addr.s6_addr),
                        u16::from_be(addr.sin6_port),
                        addr.sin6_flowinfo,
                        addr.sin6_scope_id,
                    ))
                }
                _ => {
                    unreachable!()
                }
            }
        };

        Ok((stream, addr))
    }
}

impl TcpStream {
    pub async fn connect<A: ToSocketAddrs>(addr: A) -> io::Result<TcpStream> {
        let addr = addr.to_socket_addrs()?.next().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "no address found for the given socket address",
            )
        })?;

        let socket = if addr.is_ipv4() {
            socket(libc::AF_INET, libc::SOCK_STREAM, 0).await?
        } else if addr.is_ipv6() {
            socket(libc::AF_INET6, libc::SOCK_STREAM, 0).await?
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "unsupported address family",
            ));
        };

        let op = Op::connect(socket, addr)?;
        let completion = op.await;
        if completion.result.is_err() {
            return Err(completion.result.err().unwrap());
        }
        return Ok(TcpStream { fd: socket });
    }

    pub async fn read<T: IoBufMut>(&self, buf: T) -> io::Result<(usize, T)> {
        let op = Op::recv(self.fd, buf)?;
        let mut completion = op.await;
        let result = completion.result?;
        unsafe {
            completion.data.buf.set_valid_len(result as u32);
        }
        Ok((result as usize, completion.data.buf))
    }

    pub async fn write<T: IoBuf>(&self, buf: T) -> io::Result<(usize, T)> {
        let op = Op::send(self.fd, buf)?;
        let completion = op.await;
        Ok((completion.result? as usize, completion.data.buf))
    }
}

async fn socket(domain: i32, socket_type: i32, protocol: i32) -> io::Result<RawFd> {
    let op = Op::socket(domain, socket_type, protocol)?;
    let completion = op.await;
    completion.result
}
