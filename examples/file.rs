use kunio::fs::File;
use kunio::runtime::{Runtime, spawn};
use kunio::scheduler::LocalScheduler;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let runtime = Runtime::new(Box::new(LocalScheduler), 1).expect("failed create runtime");
    let future = async {
        let file = File::create("foo.txt").await?;
        let buf: Vec<u8> = b"hello kunio!\nlove sing dance basketball!!!".to_vec();
        let (res, buf) = file.write(buf).await?;

        println!("wrote {} bytes {:?}", res, buf);

        let join_handle = spawn(async {
            let file = File::open("foo.txt").await?;
            let buf: Vec<u8> = vec![0; 12];
            let (res, buf) = file.read(buf).await?;
            println!("read {} bytes {:?}", res, buf);
            Ok::<(), Box<dyn std::error::Error>>(())
        });

        join_handle.await?;

        file.close().await?;
        Ok(())
    };

    runtime.block_on(future)
}
