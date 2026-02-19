use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    ptr::null_mut,
    sync::atomic::{AtomicBool, AtomicPtr, Ordering},
    thread,
};

/// Fair RAII spinlock with a lock-free wait queue.
/// This uses
pub struct Mutex<T: ?Sized> {
    tail: AtomicPtr<MutexQueueNode>,
    /// Recap: `UnsafeCell` is used for interior mutability, which more or
    /// less translates to "turning `&` into `&mut`". banslates
    inner: UnsafeCell<T>,
}

unsafe impl<T: ?Sized + Send> Send for Mutex<T> where T: Send {}
unsafe impl<T: ?Sized + Send> Sync for Mutex<T> where T: Send {}

struct MutexQueueNode {
    next: AtomicPtr<MutexQueueNode>,
    ready: AtomicBool,
}

impl<T> Mutex<T> {
    #[must_use]
    pub fn new(inner: T) -> Self {
        Self {
            inner: UnsafeCell::new(inner),
            tail: AtomicPtr::new(null_mut()),
        }
    }

    pub fn lock<'a>(&'a self) -> MutexGuard<'a, T> {
        self.tail.


        MutexGuard {
            mutex: &self,
            inner: unsafe { self.inner.get().as_mut().unwrap() },
        }
    }

}


#[must_use = "MutexGuard is RAII based, it will release lock immediately if dropped"]
pub struct MutexGuard<'a, T> {
    mutex: &'a Mutex<T>,
    inner: &'a mut T,
}

impl<'a, T> Deref for MutexGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<'a, T> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner
    }
}

impl<'a, T> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        self.mutex.pin.store(false, Ordering::Relaxed);
    }
}

