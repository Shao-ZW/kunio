use kunio::runtime::{Runtime, spawn};
use kunio::scheduler::LocalScheduler;

fn main() {
    let runtime = Runtime::new(Box::new(LocalScheduler));
    let res = runtime.block_on(async {
        println!("hello world!");
        let r = spawn(hello()).await;
        println!("{}", r);
        String::from("love from KUN")
    });
    println!("{}", res);
}

async fn hello() -> usize {
    println!("hello world!");
    32
}
