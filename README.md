# kitchen-sync

Implementing some concurrency primitives that I find interesting. Based mostly on [Algorithms for Scalable Synchronization on Shared-Memory Multiprocessors](https://www.cs.rochester.edu/u/scott/papers/1991_TOCS_synch.pdf). These are just using spinning and `thread::yield_now()` atm, no OS features (eg `Futex` et al).

Partly learning because I wanted to contribute an optimization based on [flat-combining](https://people.csail.mit.edu/shanir/publications/Flat%20Combining%20SPAA%2010.pdf) to an open-source LSM tree I use but experienced too many skill issues. Flat-combining is very close to the implementation of the "List based queueing lock" ("MCSLock") described in the 1991 paper.
