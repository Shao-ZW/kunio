use kunio::runtime::{Runtime, spawn_blocking};
use kunio::scheduler::LocalScheduler;

fn main() {
    let runtime = Runtime::new(Box::new(LocalScheduler), 1).expect("failed create runtime");
    let res = runtime.block_on(async {
        println!("hello world!");
        spawn_blocking(hello);
        String::from("love from KUN")
    });
    println!("{}", res);
}

fn hello() -> usize {
    std::thread::sleep(std::time::Duration::from_secs(3));
    println!("hello world!");
    32
}
