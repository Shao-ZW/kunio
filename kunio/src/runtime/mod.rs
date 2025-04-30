use crate::scheduler::{Schedule, TaskQueue};
use crate::task::{JoinHandle, dummy_waker, new_task};
use scoped_tls::scoped_thread_local;
use std::future::Future;
use std::pin::pin;
use std::task::{Context, Poll};

scoped_thread_local!(pub static RUNTIME: Runtime);

pub struct Runtime {
    pub tasks: TaskQueue,
    pub scheduler: Box<dyn Schedule>,
}

impl Runtime {
    pub fn new(scheduler: Box<dyn Schedule>) -> Self {
        Self {
            tasks: TaskQueue::new(),
            scheduler: scheduler,
        }
    }

    pub fn block_on<F>(&self, future: F) -> F::Output
    where
        F: Future,
    {
        let waker = dummy_waker();
        let cx = &mut Context::from_waker(&waker);

        RUNTIME.set(self, || {
            let join_handle = spawn(future);
            let mut join_handle = pin!(join_handle);

            loop {
                let mut max_round = self.tasks.len() * 2;
                while let Some(t) = self.tasks.pop() {
                    println!("run");
                    t.run();
                    println!("run end");
                    if max_round == 0 {
                        break;
                    } else {
                        max_round -= 1;
                    }
                }

                println!("join_handle poll");
                if let Poll::Ready(t) = join_handle.as_mut().poll(cx) {
                    return t;
                }

                // here should block for IO
            }
        })
    }
}

pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future,
{
    let (task, join_handle) = new_task(future);
    RUNTIME.with(|runtime| {
        runtime.tasks.push_back(task);
    });
    join_handle
}

pub fn spawn_blocking<F, R>(f: F) -> JoinHandle<R>
where
    F: FnOnce() -> R + 'static,
{
    todo!()
}
