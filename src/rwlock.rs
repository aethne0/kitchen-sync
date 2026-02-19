use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicU64, Ordering},
    thread,
};

/// RAII spinlock.
/// I managed to implement this without referring to anything, which was
/// surprising as I've never used `UnsafeCell` before. Divine intervention?
pub struct RwLock<T: ?Sized> {
    latch: AtomicU64,
    /// Recap: `UnsafeCell` is used for interior mutability, which more or
    /// less translates to "turning `&` into `&mut`".
    pins: UnsafeCell<T>,
}

// TODO think more about why (if) this is correct
unsafe impl<T: ?Sized + Send> Send for RwLock<T> where T: Send {}
unsafe impl<T: ?Sized + Send> Sync for RwLock<T> where T: Send {}

impl<T> RwLock<T> {
    #[must_use]
    pub fn new(inner: T) -> Self {
        Self {
            pins: UnsafeCell::new(inner),
            latch: AtomicU64::new(0),
        }
    }

    pub fn read<'a>(&'a self) -> RwLockReadGuard<'a, T> {
        // obviously this sucks

        todo!();

        RwLockReadGuard {
            mutex: &self,
            inner: unsafe { self.pins.get().as_mut().unwrap() },
        }
    }

    pub fn write<'a>(&'a self) -> RwLockReadGuard<'a, T> {
        // obviously this sucks

        todo!();

        RwLockReadGuard {
            mutex: &self,
            inner: unsafe { self.pins.get().as_mut().unwrap() },
        }
    }
}

#[must_use = "RwLockReadGuard is RAII based, it will release lock immediately if dropped"]
pub struct RwLockReadGuard<'a, T> {
    mutex: &'a RwLock<T>,
    inner: &'a mut T,
}

// impl<'a, T> MutexGuard<'a, T> {}

impl<'a, T> Deref for RwLockReadGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<'a, T> DerefMut for RwLockReadGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner
    }
}

impl<'a, T> Drop for RwLockReadGuard<'a, T> {
    fn drop(&mut self) {
        self.mutex.latch.store(todo!(), Ordering::Relaxed);
    }
}

#[must_use = "RwLockWriteGuard is RAII based, it will release lock immediately if dropped"]
pub struct RwLockWriteGuard<'a, T> {
    mutex: &'a RwLock<T>,
    inner: &'a mut T,
}

// impl<'a, T> MutexGuard<'a, T> {}

impl<'a, T> Deref for RwLockWriteGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<'a, T> DerefMut for RwLockWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner
    }
}

impl<'a, T> Drop for RwLockWriteGuard<'a, T> {
    fn drop(&mut self) {
        self.mutex.latch.store(todo!(), Ordering::Relaxed);
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
        thread::sleep(Duration::from_millis(100));
    }
}
