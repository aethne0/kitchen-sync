//! ▄ •▄ ▪  ▄▄▄▄▄ ▄▄·  ▄ .▄▄▄▄ . ▐ ▄     .▄▄ ·  ▄· ▄▌ ▐ ▄  ▄▄·
//! █▌▄▌▪██ •██  ▐█ ▌▪██▪▐█▀▄.▀·•█▌▐█    ▐█ ▀. ▐█▪██▌•█▌▐█▐█ ▌▪
//! ▐▀▀▄·▐█· ▐█.▪██ ▄▄██▀▐█▐▀▀▪▄▐█▐▐▌    ▄▀▀▀█▄▐█▌▐█▪▐█▐▐▌██ ▄▄
//! ▐█.█▌▐█▌ ▐█▌·▐███▌██▌▐▀▐█▄▄▌██▐█▌    ▐█▄▪▐█ ▐█▀·.██▐█▌▐███▌
//! ·▀  ▀▀▀▀ ▀▀▀ ·▀▀▀ ▀▀▀ · ▀▀▀ ▀▀ █▪     ▀▀▀▀   ▀ • ▀▀ █▪·▀▀▀
//! Implementing some very poor concurrency primitives.
//! This is just to learn more unsafe and "internals" stuff.
//! To be performant these would all require OS features of course
//! (futex or at least sleeping/waking threads)
//! But I want to keep these purely userspace, so just using atomics
//! and language features.
//! NAME		kitchen-sync
//!	VERSION		0
//!	HOME		https://.example.com
//!	AUTHOR		John Doe
//!	EMAIL		john AT Doe DOT net
//!	COPYRIGHT	LGPL
pub use sync::*;

mod sync {
    use std::{
        cell::UnsafeCell,
        ops::{Deref, DerefMut},
        sync::atomic::{AtomicBool, Ordering},
        thread,
    };

    /// RAII spinlock.
    /// I managed to implement this without referring to anything, which was
    /// surprising as I've never used `UnsafeCell` before. Divine intervention?
    pub struct Mutex<T: ?Sized> {
        latch: AtomicBool,
        /// Recap: `UnsafeCell` is used for interior mutability, which more or 
        /// less translates to "turning `&` into `&mut`".
        inner: UnsafeCell<T>,
    }

    // TODO: think more about why (if) this is correct
    unsafe impl<T: ?Sized + Send> Send for Mutex<T> where T: Send {}
    unsafe impl<T: ?Sized + Send> Sync for Mutex<T> where T: Send {}

    impl<T> Mutex<T> {
        #[must_use]
        pub fn new(inner: T) -> Self {
            Self {
                inner: UnsafeCell::new(inner),
                latch: AtomicBool::new(false),
            }
        }

        /// Just give her the old CMPXCHG
        pub fn lock<'a>(&'a self) -> MutexGuard<'a, T> {
            // obviously this sucks
            while self
                .latch
                .compare_exchange_weak(false, true, Ordering::Relaxed, Ordering::Relaxed)
                .is_err()
            {
                thread::yield_now();
            }

            MutexGuard {
                mutex: &self,
                inner: unsafe { self.inner.get().as_mut().unwrap() },
            }
        }

        /// The `std` version returns a `Result` because it needs to return LockPoisoned.
        /// We can just use an `Option` because it either is held or free.
        pub fn try_lock<'a>(&'a self) -> Option<MutexGuard<'a, T>> {
            if self
                .latch
                .compare_exchange_weak(false, true, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                Some(MutexGuard {
                    mutex: &self,
                    inner: unsafe { self.inner.get().as_mut().unwrap() },
                })
            } else {
                None
            }
        }

        pub fn lock_spin<'a>(&'a self) -> MutexGuard<'a, T> {
            // obviously this sucks
            while self
                .latch
                .compare_exchange_weak(false, true, Ordering::Relaxed, Ordering::Relaxed)
                .is_err()
            {}

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

    // impl<'a, T> MutexGuard<'a, T> {}

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
            self.mutex.latch.store(false, Ordering::Relaxed);
        }
    }
}

#[cfg(test)]
mod test {
    use std::{sync::Arc, thread, time::Duration};

    use crate::Mutex;

    // ya idk it seems to work boss
    #[test]
    fn test_mutex() {
        let m = Arc::new(Mutex::new(50));

        for _ in 0..4 {
            let m = m.clone();
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(50));
                let mut val = m.lock();
                println!("{}", *val);
                *val *= 2;
            });
        }

        let g = m.lock();
        thread::sleep(Duration::from_millis(500));
        drop(g);
        m.lock();
        thread::sleep(Duration::from_millis(100));
    }
}
