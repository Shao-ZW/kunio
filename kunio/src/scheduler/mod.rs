use crate::task::Task;
use std::cell::UnsafeCell;
use std::collections::VecDeque;
use std::marker::PhantomData;

pub trait Schedule {
    fn schedule(&self, task: Task);
}

pub struct TaskQueue {
    queue: UnsafeCell<VecDeque<Task>>,
    // !Send and !Sync
    _p: PhantomData<*const ()>,
}

impl TaskQueue {
    pub fn new() -> Self {
        const DEFAULT_TASK_QUEUE_SIZE: usize = 512;
        Self::new_with_capacity(DEFAULT_TASK_QUEUE_SIZE)
    }

    pub fn new_with_capacity(capacity: usize) -> Self {
        Self {
            queue: UnsafeCell::new(VecDeque::with_capacity(capacity)),
            _p: PhantomData,
        }
    }

    pub fn push_back(&self, task: Task) {
        // Safety:
        unsafe {
            (*self.queue.get()).push_back(task);
        }
    }

    pub fn push_front(&self, task: Task) {
        // Safety:
        unsafe {
            (*self.queue.get()).push_front(task);
        }
    }

    pub fn pop(&self) -> Option<Task> {
        // Safety:
        unsafe { (*self.queue.get()).pop_front() }
    }

    pub fn len(&self) -> usize {
        // Safety:
        unsafe { (*self.queue.get()).len() }
    }
}

pub struct LocalScheduler;

impl Schedule for LocalScheduler {
    fn schedule(&self, task: Task) {
        crate::runtime::RUNTIME.with(|runtime| runtime.tasks.push_back(task));
    }
}
