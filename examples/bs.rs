//! An example to show how to use TcpStream.

use kunio::net::{TcpListener, TcpStream};
use kunio::runtime::{Runtime, spawn};
use kunio::scheduler::LocalScheduler;

fn main() {
    println!("Will run with IoUringDriver(you must be on linux and enable iouring feature)");
    run();
}

fn run() {
    use futures::channel::oneshot;
    use kunio::net::{TcpListener, TcpStream};

    const ADDRESS: &str = "127.0.0.1:50000";

    let (mut tx, rx) = oneshot::channel::<()>();
    let client_thread = std::thread::spawn(|| {
        let runtime = Runtime::new(Box::new(LocalScheduler), 0).expect("failed create runtime");
        runtime.block_on(async move {
            println!("[Client] Waiting for server ready");
            tx.cancellation().await;

            println!("[Client] Server is ready, will connect and send data");
            let mut conn = TcpStream::connect(ADDRESS)
                .await
                .expect("[Client] Unable to connect to server");
            let buf: Vec<u8> = vec![97; 10];
            let (r, _) = conn.write(buf).await.unwrap();
            println!("[Client] Written {} bytes data and leave", r);
        });
    });

    let server_thread = std::thread::spawn(|| {
        let runtime = Runtime::new(Box::new(LocalScheduler), 0).expect("failed create runtime");
        runtime.block_on(async move {
            let listener = TcpListener::bind(ADDRESS)
                .unwrap_or_else(|_| panic!("[Server] Unable to bind to {ADDRESS}"));
            println!("[Server] Bind ready");
            drop(rx);

            let (mut conn, _addr) = listener
                .accept()
                .await
                .expect("[Server] Unable to accept connection");
            println!("[Server] Accepted a new connection, will read form it");

            let buf = vec![0; 64];
            let (r, buf) = conn.read(buf).await.unwrap();

            let read_len = r;
            println!(
                "[Server] Read {} bytes data: {:?}",
                read_len,
                &buf[..read_len]
            );
        });
    });

    server_thread.join().unwrap();
    client_thread.join().unwrap();
}
