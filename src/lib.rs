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

pub mod mcs_lock;
pub mod test_and_set;

// ▄▄▄▄▄▄▄▄ ..▄▄ · ▄▄▄▄▄.▄▄ ·
// •██  ▀▄.▀·▐█ ▀. •██  ▐█ ▀.
//  ▐█.▪▐▀▀▪▄▄▀▀▀█▄ ▐█.▪▄▀▀▀█▄
//  ▐█▌·▐█▄▄▌▐█▄▪▐█ ▐█▌·▐█▄▪▐█
//  ▀▀▀  ▀▀▀  ▀▀▀▀  ▀▀▀  ▀▀▀▀

#[cfg(test)]
mod test {
    use std::{
        collections::HashSet,
        sync::{mpsc, Arc, Barrier},
        thread,
    };

    use crate::mcs_lock::MCSLock;

    #[test]
    fn test_mcs() {
        const COUNT: usize = 10_000;

        let rx = {
            let (tx, rx) = mpsc::channel();

            let mcs = Arc::new(MCSLock::new(0));
            let wg = Arc::new(Barrier::new(COUNT));

            for _ in 0..COUNT {
                let mcs = mcs.clone();
                let wg = wg.clone();
                let tx = tx.clone();

                thread::spawn(move || {
                    wg.wait();

                    let mut val_l = mcs.lock();
                    tx.send(*val_l).unwrap();
                    *val_l += 1;
                });
            }
            rx
        };

        let mut rcvd = HashSet::with_capacity(COUNT);

        while let Ok(val) = rx.recv() {
            rcvd.insert(val);
        }

        for i in 0..COUNT {
            assert!(rcvd.contains(&i));
        }
    }
}
