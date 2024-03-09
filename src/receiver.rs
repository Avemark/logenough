use std::sync::atomic::{AtomicBool, Ordering};

use crate::logdata::LogData;
use crate::logline::Logline;
use parking_lot::MutexGuard;

struct Receiver<const N: usize> {
    position: usize,
    data: LogData<N>,
}

impl<const N: usize> Receiver<N> {
    fn receive<F>(mut self, interrupted: &AtomicBool, f: F)
    where
        F: Fn(&Logline),
    {
        while !interrupted.load(Ordering::Relaxed) {
            //{
            //    let mut reference = self.reference_lock();
            //    if self.position == *reference {
            //        self.data.cond.wait(&mut reference);
            //    }
            // }
            let reference = *self.reference_lock();
            while self.position < reference {
                if interrupted.load(Ordering::Relaxed) {
                    break;
                }
                f(&self.data.data[self.increment()].lock());
            }
            self.data.cond.wait(&mut self.reference_lock())
        }
    }

    fn reference_lock(&self) -> MutexGuard<usize> {
        self.data.reference.lock()
    }

    fn increment(&mut self) -> usize {
        self.position += 1;
        if self.position >= N {
            self.position = 0;
        }
        self.position
    }
}