//! An echo example. From monoio
//!
//! Run the example and `nc 127.0.0.1 50002` in another shell.
//! All your input will be echoed out.

use kunio::net::{TcpListener, TcpStream};
use kunio::runtime::{Runtime, spawn};
use kunio::scheduler::LocalScheduler;

fn main() {
    let runtime = Runtime::new(Box::new(LocalScheduler), 0).expect("failed create runtime");
    runtime.block_on(async {
        let listener = TcpListener::bind("127.0.0.1:50002").unwrap();
        println!("listening");
        loop {
            let incoming = listener.accept().await;
            match incoming {
                Ok((stream, addr)) => {
                    println!("accepted a connection from {addr}");
                    spawn(echo(stream));
                }
                Err(e) => {
                    println!("accepted connection failed: {e}");
                    return;
                }
            }
        }
    });
}

async fn echo(stream: TcpStream) -> std::io::Result<()> {
    let mut buf: Vec<u8> = vec![0; 4096];
    let mut res;
    loop {
        (res, buf) = stream.read(buf).await?;
        if res == 0 {
            return Ok(());
        }

        (res, buf) = stream.write(buf).await?;
    }
}
