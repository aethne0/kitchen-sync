# kitchen-sync
Implementing some concurrency primitives that I find interesting. Based mostly on [Algorithms for Scalable Synchronization on Shared-Memory Multiprocessors](https://www.cs.rochester.edu/u/scott/papers/1991_TOCS_synch.pdf). These are just using spinning and `thread::yield_now()` atm, no OS features (eg `Futex` et al).
