use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::future::Future;
use std::io;
use std::pin::pin;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;
use std::task::{Context, Poll};

use crossbeam::queue::SegQueue;
use lazy_static::lazy_static;
use scoped_tls::scoped_thread_local;
use threadpool::ThreadPool;

use crate::driver::UringDriver;
use crate::scheduler::{Schedule, TaskQueue};
use crate::task::{BlockingFuture, JoinHandle, Task, dummy_waker, new_blocking_task, new_task};

lazy_static! {
    pub static ref RUNTIME_EXT: RuntimeExtCollection = RuntimeExtCollection::new();
}

pub static RUNTIME_IDGEN: AtomicU32 = AtomicU32::new(0);

scoped_thread_local!(pub static RUNTIME: Runtime);

pub struct RuntimeExtCollection {
    inner: UnsafeCell<HashMap<u32, RuntimeExt>>,
}

// Safety:
unsafe impl Sync for RuntimeExtCollection {}

impl RuntimeExtCollection {
    pub fn new() -> Self {
        Self {
            inner: UnsafeCell::new(HashMap::new()),
        }
    }

    pub fn insert(&self, id: u32, runtime_ext: RuntimeExt) {
        unsafe { (*self.inner.get()).insert(id, runtime_ext) };
    }

    pub fn get(&self, id: u32) -> Option<&RuntimeExt> {
        unsafe { (*self.inner.get()).get(&id) }
    }
}

pub struct RuntimeExt {
    task_count: AtomicU32,
    woken_tasks: SegQueue<Task>,
}

impl RuntimeExt {
    pub fn new() -> Self {
        Self {
            task_count: AtomicU32::new(0),
            woken_tasks: SegQueue::new(),
        }
    }

    pub fn task_count(&self) -> u32 {
        self.task_count.load(Ordering::Relaxed)
    }

    pub fn woken_count(&self) -> u32 {
        self.woken_tasks.len() as u32
    }

    pub fn fetch_add_count(&self, val: u32) -> u32 {
        self.task_count.fetch_add(val, Ordering::Relaxed)
    }

    pub fn fetch_sub_count(&self, val: u32) -> u32 {
        self.task_count.fetch_sub(val, Ordering::Relaxed)
    }

    pub fn push_woken_tasks(&self, task: Task) {
        self.woken_tasks.push(task);
    }

    pub fn pop_woken_tasks(&self) -> Option<Task> {
        self.woken_tasks.pop()
    }
}

pub struct Runtime {
    pub tasks: TaskQueue,
    pub scheduler: Box<dyn Schedule>,
    pub driver: UringDriver,
    pub threadpool: Option<ThreadPool>,
    pub id: u32,
}

impl Runtime {
    pub fn new(scheduler: Box<dyn Schedule>, attach_thread_size: usize) -> io::Result<Self> {
        let id = RUNTIME_IDGEN.fetch_add(1, Ordering::Relaxed);
        RUNTIME_EXT.insert(id, RuntimeExt::new());

        Ok(Self {
            tasks: TaskQueue::new(),
            scheduler: scheduler,
            driver: UringDriver::new()?,
            threadpool: if attach_thread_size == 0 {
                None
            } else {
                Some(ThreadPool::new(attach_thread_size))
            },
            id,
        })
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
                // avoid starving
                let mut max_round = self.tasks.len() * 2;
                while let Some(t) = self.tasks.pop() {
                    t.run();
                    if max_round == 0 {
                        break;
                    } else {
                        max_round -= 1;
                    }
                }

                let mut max_round = RUNTIME_EXT.get(self.id).unwrap().woken_count() * 2;
                while let Some(t) = RUNTIME_EXT.get(self.id).unwrap().pop_woken_tasks() {
                    t.run();
                    if max_round == 0 {
                        break;
                    } else {
                        max_round -= 1;
                    }
                }

                if RUNTIME_EXT.get(self.id).unwrap().task_count() == 0 {
                    if let Poll::Ready(t) = join_handle.as_mut().poll(cx) {
                        return t;
                    }
                }

                let _ = self.driver.submit_and_wait();
            }
        })
    }
}

pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future,
{
    RUNTIME.with(|runtime| {
        let (task, join_handle) = new_task(future, runtime.id);
        RUNTIME_EXT.get(runtime.id).unwrap().fetch_add_count(1);
        runtime.tasks.push_back(task);
        join_handle
    })
}

pub fn spawn_blocking<F, R>(f: F) -> JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    RUNTIME.with(|runtime| {
        let (task, join_handle) = new_blocking_task(BlockingFuture(Some(f)), runtime.id);
        match runtime.threadpool {
            Some(ref pool) => {
                RUNTIME_EXT.get(runtime.id).unwrap().fetch_add_count(1);
                pool.execute(move || {
                    task.run();
                });
            }
            None => panic!("threadpool is empty"),
        }
        join_handle
    })
}
