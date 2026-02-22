# kitchen-sync
[![Rust](https://github.com/aethne0/kitchen-sync/actions/workflows/rust.yml/badge.svg)](https://github.com/aethne0/kitchen-sync/actions/workflows/rust.yml)

[Implementing](https://github.com/aethne0/kitchen-sync/blob/master/src/mcs_lock.rs) RAII Mellor-Crummey-Scott lock, as well as others. Only implementing for linux atm.

Implementing some concurrency primitives that I find interesting. Based mostly on [Algorithms for Scalable Synchronization on Shared-Memory Multiprocessors](https://www.cs.rochester.edu/u/scott/papers/1991_TOCS_synch.pdf). These are just using spinning and `thread::yield_now()`/`std::hint::spin_loop()` atm, no OS features (eg `Futex` et al).

Partly learning because I wanted to contribute an optimization based on [flat-combining](https://people.csail.mit.edu/shanir/publications/Flat%20Combining%20SPAA%2010.pdf) to an open-source LSM tree I use but experienced too many skill issues. Flat-combining is very close to the implementation of the "List based queueing lock" ("MCSLock") described in the 1991 paper.
