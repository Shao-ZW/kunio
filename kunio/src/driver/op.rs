use std::io;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};

use crate::runtime::RUNTIME;

mod close;
mod open;
mod read;
mod write;

pub struct Op<T> {
    pub id: u64,
    data: Option<T>,
}

pub struct Completion<T> {
    pub data: T,
    pub result: io::Result<i32>,
}

pub enum OpStage {
    Submitted,
    Waiting(Waker),
    Completed(i32),
}

pub trait UringOp {
    fn build_sqe(&mut self) -> io_uring::squeue::Entry;
}

impl<T: UringOp> Op<T> {
    pub fn new(id: u64, data: T) -> Self {
        Self {
            id,
            data: Some(data),
        }
    }

    pub fn build_sqe(&mut self) -> io_uring::squeue::Entry {
        match self.data {
            Some(ref mut data) => data.build_sqe().user_data(self.id),
            None => unsafe { std::hint::unreachable_unchecked() },
        }
    }

    pub fn poll(&mut self, cx: &mut Context<'_>) -> Poll<io::Result<i32>> {
        RUNTIME.with(|runtime| runtime.driver.poll_op(self, cx))
    }
}

impl<T: UringOp> Future for Op<T> {
    type Output = Completion<T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Safety:
        let op = unsafe { self.get_unchecked_mut() };

        match op.poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(result) => {
                let data = op.data.take().unwrap();
                Poll::Ready(Completion { data, result })
            }
        }
    }
}
