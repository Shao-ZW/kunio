use crate::utils::IdGenerator;
use io_uring::IoUring;
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::io;
use std::task::{Context, Poll};

pub mod op;

use op::*;

pub struct UringDriver {
    inner: UnsafeCell<UringInner>,
}

impl UringDriver {
    pub fn new() -> io::Result<Self> {
        let inner = UringInner::new()?;
        Ok(Self {
            inner: UnsafeCell::new(inner),
        })
    }

    pub fn submit_op<T: UringOp>(&self, data: T) -> io::Result<Op<T>> {
        unsafe { (*self.inner.get()).submit_op(data) }
    }

    pub fn submit_and_wait(&self) -> io::Result<()> {
        unsafe { (*self.inner.get()).submit_and_wait() }
    }

    pub fn poll_op<T: UringOp>(
        &self,
        op: &mut Op<T>,
        cx: &mut Context<'_>,
    ) -> Poll<io::Result<i32>> {
        unsafe { (*self.inner.get()).poll_op(op, cx) }
    }
}

struct UringInner {
    ops: HashMap<u64, OpStage>,
    uring: IoUring,
    id_generator: IdGenerator,
    waiting: usize,
}

impl UringInner {
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            ops: HashMap::new(),
            uring: IoUring::new(100)?,
            id_generator: IdGenerator::new(),
            waiting: 0,
        })
    }

    fn submit_sync(&mut self) -> io::Result<()> {
        self.uring.submit()?;
        Ok(())
    }

    fn complete_sync(&mut self) -> io::Result<()> {
        let cq = self.uring.completion();
        for cqe in cq {
            let id = cqe.user_data();
            let result = cqe.result();
            self.waiting -= 1;

            if let Some(op_stage) = self.ops.get_mut(&id) {
                match op_stage {
                    OpStage::Submitted => {
                        *op_stage = OpStage::Completed(result);
                    }
                    OpStage::Waiting(waker) => {
                        // This is ok because our runtime is single thread
                        waker.wake_by_ref();
                        *op_stage = OpStage::Completed(result);
                    }
                    OpStage::Completed(_) => unsafe {
                        std::hint::unreachable_unchecked();
                    },
                }
            }
        }
        Ok(())
    }

    // This is not a real submit like in io_uring
    fn submit_op<T: UringOp>(&mut self, data: T) -> io::Result<Op<T>> {
        let id = self.id_generator.gen_id();
        let mut op = Op::new(id, data);

        if self.uring.submission().is_full() {
            self.submit_sync()?;
        }

        self.ops.insert(id, OpStage::Submitted);

        self.waiting += 1;
        let sqe = op.build_sqe();
        unsafe {
            self.uring.submission().push(&sqe).unwrap();
        }
        Ok(op)
    }

    fn submit_and_wait(&mut self) -> io::Result<()> {
        if self.waiting > 0 {
            self.uring.submit_and_wait(1)?;
            self.complete_sync()?;
        }
        Ok(())
    }

    fn poll_op<T: UringOp>(
        &mut self,
        op: &mut Op<T>,
        cx: &mut Context<'_>,
    ) -> Poll<io::Result<i32>> {
        let id = op.id;
        match self.ops.get_mut(&id) {
            Some(op_stage) => match op_stage {
                OpStage::Submitted => {
                    *op_stage = OpStage::Waiting(cx.waker().clone());
                    return Poll::Pending;
                }
                OpStage::Waiting(waker) => {
                    if !waker.will_wake(cx.waker()) {
                        *waker = cx.waker().clone();
                    }
                    return Poll::Pending;
                }
                _ => {}
            },
            None => panic!(),
        }

        match self.ops.remove(&id) {
            Some(op_stage) => match op_stage {
                OpStage::Completed(result) => {
                    let result = if result < 0 {
                        Err(io::Error::from_raw_os_error(-result))
                    } else {
                        Ok(result)
                    };
                    Poll::Ready(result)
                }
                _ => {
                    unreachable!("unexpected stage!")
                }
            },
            None => panic!(),
        }
    }
}
