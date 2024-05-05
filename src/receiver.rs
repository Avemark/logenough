use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::logdata::LogData;
use crate::logline::Logline;
use parking_lot::MutexGuard;

pub struct Receiver<'a, const N: usize> {
    position: usize,
    data: &'a Arc<LogData<N>>,
}

const ORDERING: Ordering = Ordering::SeqCst;

impl<'a, const N: usize> Receiver<'a, N> {
    pub fn receive<F>(mut self, interrupted: &AtomicBool, f: F)
    where
        F: Fn(&Logline),
    {
        #[cfg(debug_assertions)]
        println!("Receiver waiting for initial notice");
        self.data.cond.wait(&mut self.reference_lock());
        #[cfg(debug_assertions)]
        println!("Received initial notice");
        if interrupted.load(ORDERING) {
            #[cfg(debug_assertions)]
            println!("Interrupted, exiting without processing");
            return;
        }
        loop {
            let upcoming_read = {
                let mut reflock = self.reference_lock();
                if self.position == *reflock {
                    #[cfg(debug_assertions)]
                    println!("waiting for condvar");
                    self.data.cond.wait(&mut reflock);
                }
                *reflock
            };
            #[cfg(debug_assertions)]
            println!(
                "Receiver working, position {}, trying to catch up to the upcoming read at {}",
                self.position, upcoming_read
            );
            while self.position != upcoming_read {
                #[cfg(debug_assertions)]
                println!("Receiver reading at {}", self.position);
                f(&self.data.data[self.position].lock());
                self.increment();
            }
            if interrupted.load(ORDERING) {
                #[cfg(debug_assertions)]
                println!("Receiver interrupted, quitting");
                return;
            }
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

    pub fn new(logdata: &'a Arc<LogData<N>>) -> Self {
        Self {
            position: 0usize,
            data: logdata,
        }
    }
}

#[cfg(test)]
mod receiver_test {
    use super::Receiver;
    use crate::logdata::LogData;
    use std::sync::{atomic::AtomicBool, Arc};

    #[test]
    fn test_receive() {
        let logdata = Arc::new(LogData::<5>::new());
        let receiver = Receiver::new(&logdata);

        let func: fn(&mut [u8]) -> Result<usize, ()> = |buffer: &mut [u8]| {
            for (index, byte) in "Hello".as_bytes().iter().enumerate() {
                buffer[index] = *byte;
            }
            Ok("Hello".len())
        };

        logdata.receive(func).unwrap();
        logdata.receive(func).unwrap();

        assert_eq!(
            format!("{}", "Hello"),
            format!("{}", logdata.data[0].lock())
        );

        let interrupted = AtomicBool::new(true);

        receiver.receive(&interrupted, |logline| {
            assert_eq!(format!("{}", "Hello"), format!("{}", logline));
        })
    }
}
