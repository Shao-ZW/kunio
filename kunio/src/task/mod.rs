use std::cell::{Cell, UnsafeCell};
use std::future::Future;
use std::marker::PhantomData;
use std::mem;
use std::pin::Pin;
use std::ptr::NonNull;
use std::task::{Context, Poll, Waker};

pub mod waker;

pub use waker::*;

use crate::runtime;

pub struct Task {
    raw: RawTask,
}

impl Task {
    pub fn new(raw: RawTask) -> Self {
        raw.header().ref_inc();
        Self { raw }
    }

    pub fn run(self) {
        self.raw.poll();
    }
}

impl Drop for Task {
    fn drop(&mut self) {
        self.raw.drop_ref();
    }
}

pub struct JoinHandle<T> {
    raw: RawTask,
    _p: PhantomData<T>,
}

impl<T> JoinHandle<T> {
    pub fn new(raw: RawTask) -> Self {
        raw.header().ref_inc();
        Self {
            raw,
            _p: PhantomData,
        }
    }
}

impl<T> Drop for JoinHandle<T> {
    fn drop(&mut self) {
        self.raw.drop_ref();
    }
}

impl<T> Future for JoinHandle<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut ret = Poll::Pending;
        // Safety:
        unsafe {
            self.raw
                .try_read_output(&mut ret as *mut _ as *mut (), cx.waker());
        }

        ret
    }
}

#[derive(Clone, Copy)]
pub struct RawTask {
    ptr: NonNull<Header>,
}

impl RawTask {
    pub fn new<F: Future>(future: F) -> Self {
        let ptr = Box::into_raw(TaskEntity::new(future));
        // Safety:
        let ptr = unsafe { NonNull::new_unchecked(ptr as *mut Header) };
        Self { ptr }
    }

    pub fn header(&self) -> &Header {
        // Safety:
        unsafe { self.ptr.as_ref() }
    }

    pub fn poll(self) {
        let vtable = self.header().vtable;
        // Safety:
        unsafe { (vtable.poll)(self.ptr) }
    }

    pub unsafe fn try_read_output(self, res: *mut (), waker: &Waker) {
        let vtable = self.header().vtable;
        // Safety:
        unsafe { (vtable.try_read_output)(self.ptr, res, waker) }
    }

    pub fn drop_ref(self) {
        let vtable = self.header().vtable;
        // Safety:
        unsafe { (vtable.drop_ref)(self.ptr) }
    }
}

pub struct Header {
    refcount: Cell<usize>,
    vtable: &'static Vtable,
}

impl Header {
    pub fn new<F: Future>() -> Self {
        Self {
            refcount: Cell::new(0),
            vtable: vtable::<F>(),
        }
    }

    fn ref_dec(&self) -> usize {
        let cnt = self.refcount.get();
        println!(
            "KUNIO DEBUG[State]: ref_dec {} -> {}, ptr: {:p}",
            cnt,
            cnt - 1,
            self
        );
        self.refcount.set(cnt - 1);
        cnt - 1
    }

    fn ref_inc(&self) -> usize {
        let cnt = self.refcount.get();
        println!(
            "KUNIO DEBUG[State]: ref_inc {} -> {}, ptr: {:p}",
            cnt,
            cnt + 1,
            self
        );
        self.refcount.set(cnt + 1);
        cnt + 1
    }
}

pub enum Stage<F: Future> {
    Runnable(F),
    Pending(F),
    Finished(F::Output),
    Consumed,
}

pub struct Vtable {
    pub poll: unsafe fn(NonNull<Header>),
    pub try_read_output: unsafe fn(NonNull<Header>, *mut (), &Waker),
    pub drop_ref: unsafe fn(NonNull<Header>),
}

pub fn vtable<F: Future>() -> &'static Vtable {
    &Vtable {
        poll: poll::<F>,
        try_read_output: try_read_output::<F>,
        drop_ref: drop_ref::<F>,
    }
}

unsafe fn poll<F: Future>(ptr: NonNull<Header>) {
    let handle = TaskHandle::<F>::from_raw(ptr);
    handle.poll();
}

unsafe fn try_read_output<F: Future>(ptr: NonNull<Header>, res: *mut (), waker: &Waker) {
    let handle = TaskHandle::<F>::from_raw(ptr);
    let res = unsafe { &mut *(res as *mut Poll<F::Output>) };
    handle.try_read_output(res, waker);
}

unsafe fn drop_ref<F: Future>(ptr: NonNull<Header>) {
    let handle = TaskHandle::<F>::from_raw(ptr);
    handle.drop_ref();
}

struct Core<F: Future> {
    stage: UnsafeCell<Stage<F>>,
}

impl<F: Future> Core<F> {
    pub fn new(future: F) -> Self {
        Self {
            stage: UnsafeCell::new(Stage::Runnable(future)),
        }
    }

    // only called the task in taskqueue(Runnable)
    pub fn poll(&self, cx: &mut Context<'_>) -> Poll<()> {
        let res = match unsafe { &mut *self.stage.get() } {
            Stage::Runnable(future) => {
                let future = unsafe { Pin::new_unchecked(future) };
                future.poll(cx)
            }
            _ => {
                unreachable!("unexpected stage!")
            }
        };

        if let Poll::Ready(output) = res {
            // Safety:
            unsafe {
                *self.stage.get() = Stage::Finished(output);
            }
            return Poll::Ready(());
        }
        Poll::Pending
    }

    pub fn try_read_output(&self, res: &mut Poll<F::Output>) {
        if let Stage::Finished(_) = unsafe { &*self.stage.get() } {
            println!("reach!");
            match mem::replace(unsafe { &mut *self.stage.get() }, Stage::Consumed) {
                Stage::Finished(output) => *res = Poll::Ready(output),
                _ => unreachable!(),
            }
        }
    }
}

pub struct Trailer {
    join_waker: UnsafeCell<Option<Waker>>,
}

impl Trailer {
    fn has_join_waker(&self) -> bool {
        unsafe { (*self.join_waker.get()).is_some() }
    }

    fn set_waker(&self, waker: &Waker) {
        // Safety:
        unsafe {
            if let Some(join_waker) = &(*self.join_waker.get()) {
                if join_waker.will_wake(waker) {
                    return;
                }
            }
            *self.join_waker.get() = Some(waker.clone());
        }
    }

    fn join_wake(&self) {
        match unsafe { &*self.join_waker.get() } {
            Some(waker) => waker.wake_by_ref(),
            None => unreachable!(),
        }
    }
}

// only live in heap
pub struct TaskEntity<F: Future> {
    header: Header,
    core: Core<F>,
    trailer: Trailer,
}

impl<F: Future> TaskEntity<F> {
    fn new(future: F) -> Box<Self> {
        Box::new(TaskEntity {
            header: Header::new::<F>(),
            core: Core::new(future),
            trailer: Trailer {
                join_waker: UnsafeCell::new(None),
            },
        })
    }
}

pub struct TaskHandle<F: Future> {
    task: NonNull<TaskEntity<F>>,
}

impl<F: Future> TaskHandle<F> {
    fn from_raw(ptr: NonNull<Header>) -> Self {
        Self {
            task: ptr.cast::<TaskEntity<F>>(),
        }
    }

    fn has_join_waker(&self) -> bool {
        self.trailer().has_join_waker()
    }

    fn header(&self) -> &Header {
        unsafe { &self.task.as_ref().header }
    }

    fn core(&self) -> &Core<F> {
        unsafe { &self.task.as_ref().core }
    }

    fn trailer(&self) -> &Trailer {
        unsafe { &self.task.as_ref().trailer }
    }

    fn poll(self) {
        println!("real poll");
        let waker = unsafe { Waker::from_raw(raw_waker::<F>(self.header())) };
        let mut cx = Context::from_waker(&waker);
        if let Poll::Ready(_) = self.core().poll(&mut cx) {
            if self.has_join_waker() {
                self.trailer().join_wake();
            }
        }
        println!("real poll end");
    }

    fn try_read_output(self, res: &mut Poll<F::Output>, waker: &Waker) {
        self.trailer().set_waker(waker);
        self.core().try_read_output(res);
    }

    fn dealloc(self) {
        // Safety:
        unsafe {
            drop(Box::from_raw(self.task.as_ptr()));
        }
    }

    fn drop_ref(self) {
        if self.header().ref_dec() == 0 {
            self.dealloc();
        }
    }

    fn get_new_task(&self) -> Task {
        println!("get new task!");
        Task::new(RawTask {
            ptr: self.task.cast(),
        })
    }

    fn wake_by_ref(&self) {
        runtime::RUNTIME.with(|runtime| {
            runtime.scheduler.schedule(self.get_new_task());
        });
    }

    fn wake_by_val(self) {
        runtime::RUNTIME.with(|runtime| {
            runtime.scheduler.schedule(self.get_new_task());
        });
    }
}

pub fn new_task<F: Future>(future: F) -> (Task, JoinHandle<F::Output>) {
    let raw = RawTask::new(future);
    (Task::new(raw), JoinHandle::new(raw))
}
