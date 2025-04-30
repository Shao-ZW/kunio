use crate::task::{Header, TaskHandle};
use std::ptr::NonNull;
use std::task::{RawWaker, RawWakerVTable, Waker};

unsafe fn clone_waker<F: Future>(ptr: *const ()) -> RawWaker {
    let header = ptr as *const Header;
    unsafe { raw_waker::<F>(&(*header)) }
}

unsafe fn drop_waker<F: Future>(ptr: *const ()) {
    let ptr = unsafe { NonNull::new_unchecked(ptr as *mut Header) };
    let handle = TaskHandle::<F>::from_raw(ptr);
    handle.drop_ref();
}

unsafe fn wake_by_val<F: Future>(ptr: *const ()) {
    let ptr = unsafe { NonNull::new_unchecked(ptr as *mut Header) };
    let handle = TaskHandle::<F>::from_raw(ptr);
    handle.wake_by_val();
}

unsafe fn wake_by_ref<F: Future>(ptr: *const ()) {
    let ptr = unsafe { NonNull::new_unchecked(ptr as *mut Header) };
    let handle = TaskHandle::<F>::from_raw(ptr);
    handle.wake_by_ref();
}

pub fn raw_waker<F: Future>(header: &Header) -> RawWaker {
    let ptr = header as *const _ as *const ();
    let vtable = &RawWakerVTable::new(
        clone_waker::<F>,
        wake_by_val::<F>,
        wake_by_ref::<F>,
        drop_waker::<F>,
    );
    header.ref_inc();
    RawWaker::new(ptr, vtable)
}

pub fn dummy_waker() -> Waker {
    fn raw_waker() -> RawWaker {
        RawWaker::new(std::ptr::null::<()>(), vtable())
    }

    fn vtable() -> &'static RawWakerVTable {
        &RawWakerVTable::new(|_| raw_waker(), |_| {}, |_| {}, |_| {})
    }

    unsafe { Waker::from_raw(raw_waker()) }
}
