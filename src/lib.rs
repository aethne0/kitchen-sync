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
//! and language features and such.

mod mutex_queued;
pub use mutex_queued::Mutex;
