use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, Ordering},
    thread,
};


/// Fair RAII spinlock with a lock-free wait queue.
/// This uses 
pub struct MutexFair<T: ?Sized> {
    pin: AtomicBool,
    /// Recap: `UnsafeCell` is used for interior mutability, which more or
    /// less translates to "turning `&` into `&mut`". banslates
    inner: UnsafeCell<T>,
}

unsafe impl<T: ?Sized + Send> Send for MutexFair<T> where T: Send {}
unsafe impl<T: ?Sized + Send> Sync for MutexFair<T> where T: Send {}

impl<T> MutexFair<T> {
    #[must_use]
    pub fn new(inner: T) -> Self {
        Self {
            inner: UnsafeCell::new(inner),
            head: Atomic
            pin: AtomicBool::new(false),
        }
    }

    /// Just give her the old CMPXCHG
    pub fn lock<'a>(&'a self) -> MutexFairGuard<'a, T> {
        // obviously this sucks
        while self
            .pin
            .compare_exchange_weak(false, true, Ordering::Relaxed, Ordering::Relaxed)
            .is_err()
        {
            thread::yield_now();
        }

        MutexFairGuard {
            mutex: &self,
            inner: unsafe { self.inner.get().as_mut().unwrap() },
        }
    }

    /// The `std` version returns a `Result` because it needs to return LockPoisoned.
    /// We can just use an `Option` because it either is held or free.
    pub fn try_lock<'a>(&'a self) -> Option<MutexFairGuard<'a, T>> {
        if self
            .pin
            .compare_exchange_weak(false, true, Ordering::Relaxed, Ordering::Relaxed)
            .is_ok()
        {
            Some(MutexFairGuard {
                mutex: &self,
                inner: unsafe { self.inner.get().as_mut().unwrap() },
            })
        } else {
            None
        }
    }

    pub fn lock_spin<'a>(&'a self) -> MutexFairGuard<'a, T> {
        while self
            .pin
            .compare_exchange_weak(false, true, Ordering::Relaxed, Ordering::Relaxed)
            .is_err()
        {}

        MutexFairGuard {
            mutex: &self,
            inner: unsafe { self.inner.get().as_mut().unwrap() },
        }
    }
}

#[must_use = "MutexFairGuard is RAII based, it will release lock immediately if dropped"]
pub struct MutexFairGuard<'a, T> {
    mutex: &'a MutexFair<T>,
    inner: &'a mut T,
}

// impl<'a, T> MutexFairGuard<'a, T> {}

impl<'a, T> Deref for MutexFairGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<'a, T> DerefMut for MutexFairGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner
    }
}

impl<'a, T> Drop for MutexFairGuard<'a, T> {
    fn drop(&mut self) {
        self.mutex.pin.store(false, Ordering::Relaxed);
    }
}

