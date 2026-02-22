#![cfg(target_os = "linux")]
use std::{
    cell::{RefCell, UnsafeCell},
    ops::{Deref, DerefMut},
    ptr::null_mut,
    sync::atomic::{AtomicBool, AtomicPtr, Ordering},
};

/// Simple RAII Mellor-Crummey-Scott lock
pub struct MCSLock<T: ?Sized> {
    tail: AtomicPtr<QueueNode>,
    inner: UnsafeCell<T>,
}

const SPINS_LOCK: usize = 64; // value chosen by consulting celestial alignments

unsafe impl<T: ?Sized + Send> Send for MCSLock<T> where T: Send {}
unsafe impl<T: ?Sized + Send> Sync for MCSLock<T> where T: Send {}

/// This is a thread-local struct used by threads to attach themselves into the "queue" link-list.
/// Threads will attach themselves at the `tail` of the lock which is "last in line".
/// Threads will spin on their own `locked` field, which will be eventually set to `false` by the
/// thread "ahead of them in line" (in other words: the thread that gets the lock immediately before them).
#[repr(align(64))]
struct QueueNode {
    next: AtomicPtr<QueueNode>,
    locked: AtomicBool,
}

thread_local!(
    static QUEUE_NODE: RefCell<QueueNode> = const {
        RefCell::new(QueueNode {
            locked: AtomicBool::new(false),
            next: AtomicPtr::new(null_mut()),
        })
    };
);

impl<T> MCSLock<T> {
    #[must_use]
    pub fn new(inner: T) -> Self {
        Self {
            inner: UnsafeCell::new(inner),
            tail: AtomicPtr::new(null_mut()),
        }
    }

    pub fn lock<'a>(&'a self) -> Guard<'a, T> {
        QUEUE_NODE.with_borrow_mut(|queue_node| {
            // I think this is needed, otherwise we may have a stale `next` ptr
            queue_node.next.store(null_mut(), Ordering::Relaxed);

            // ptr to our thread_local queue_node struct
            let queue_node_ptr = (queue_node) as *mut _;

            // This will be the queue_node "before" us temporally speaking, if any.
            // This node will point to us as the "next in line".
            // We are now pointed to by the lock's tail ptr
            let pred = self.tail.swap(queue_node_ptr, Ordering::AcqRel);

            // If there was indeed a predecessor in the queue, then we do the following:
            if !pred.is_null() {
                // 1. Set ourselves as "locked" (it is no longer our turn)
                queue_node.locked.store(true, Ordering::Relaxed);
                // 2. Set the predecessor to point to us as "next in line"
                //      NOTE: Its possible here that we now have `locked` ourselves, but haven't yet
                //      set `pred` to point to us as `next`. If `pred` then runs and `release`s, it
                //      won't know to `unlock` us because its `next` ptr is not yet set. This is handled
                //      in unlock - otherwise we'd have a race that could lead to a deadlock (we're
                //      locked and nobody will ever unlock us).
                unsafe { (*pred).next.store(queue_node_ptr, Ordering::Release) };
                // 3. Spin on our "locked" field until we are unlocked (which the predecessor will do
                //    once it releases the lock)
                let mut spin_cnt = 0;
                while queue_node.locked.load(Ordering::Acquire) {
                    if spin_cnt < SPINS_LOCK {
                        spin_cnt += 1;
                        std::hint::spin_loop();
                    } else {
                        unsafe {
                            libc::syscall(
                                libc::SYS_futex,
                                &queue_node.locked as *const AtomicBool,
                                libc::FUTEX_WAIT | libc::FUTEX_PRIVATE_FLAG,
                                1,
                                std::ptr::null::<libc::timespec>(),
                            );
                        }
                    }
                }
            }
            // If there was nobody in the queue, it is our turn instantly - we simply grab the lock

            // We are pointed to by the tail ptr, so if another thread joins the queue they will
            // attach themselves to us via our `next` ptr, so we can unlock them once we're done
        });

        Guard {
            lock: &self,
            inner: unsafe { self.inner.get().as_mut().unwrap() },
        }
    }
}

#[must_use = "MCSLock is RAII based, it will release lock immediately if dropped"]
pub struct Guard<'a, T> {
    lock: &'a MCSLock<T>,
    inner: &'a mut T,
}

impl<'a, T> Drop for Guard<'a, T> {
    fn drop(&mut self) {
        QUEUE_NODE.with_borrow_mut(|queue_node| {
            // ptr to our thread_local queue_node
            let queue_node_ptr = queue_node as *mut _;

            // Has anyone attached themselves "after" us while we were working?
            while queue_node.next.load(Ordering::Acquire).is_null() {
                // We cmpxchg the lock's `tail` ptr with `null`, if this fails it means that
                // someone has added themselves to the queue since we did our `next == null` check
                // in the while condition immediately before this.
                //
                // Regarding the note in `lock()`:
                // In `lock` the `tail` ptr is swapped before the predecessor's `next` ptr is set,
                // so either of the two following things can happen:
                // 1. later-thread did `swap` on tail and got back `null`, and didn't bother to
                //    `lock` itself, so we're fine.
                // 2. later-thread did `swap` on tail and got back our queue_node, in which case
                //    the following cmpxchg will fail until we can see the later-thread has gotten
                //    around to updating our `next` ptr
                if self
                    .lock
                    .tail
                    .compare_exchange(
                        queue_node_ptr,
                        null_mut(),
                        Ordering::Release, // TODO: think about ordering more carefully
                        Ordering::Relaxed,
                    )
                    .is_ok()
                {
                    // if we succeeded then there is indeed nobody after us in the line, so
                    // there's nothing left for us to do.
                    return;
                }
                // If our cmpxchg failed we check our queue_node's `next` ptr again to see the node
                // that must have been added to the list after us.

                std::hint::spin_loop();
            }

            // If someone *is* after us, and we now have a ptr to their queue_node, we simply need to
            // unlock them now, after that our work is done.
            unsafe {
                let locked_field = &(*queue_node.next.load(Ordering::Relaxed)).locked;
                locked_field.store(false, Ordering::Release);

                // wake if slept
                libc::syscall(
                    libc::SYS_futex,
                    locked_field as *const AtomicBool,
                    libc::FUTEX_WAKE | libc::FUTEX_PRIVATE_FLAG,
                    1,
                );
            }
        });
    }
}

impl<'a, T> Deref for Guard<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<'a, T> DerefMut for Guard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner
    }
}
