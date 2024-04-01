use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::logdata::LogData;
use crate::logline::Logline;
use parking_lot::MutexGuard;

pub struct Receiver<const N: usize> {
    position: usize,
    data: Arc<LogData<N>>,
}

#[cfg(test)]
const ORDERING: Ordering = Ordering::SeqCst;

#[cfg(not(test))]
const ORDERING: Ordering = Ordering::Relaxed;

impl<const N: usize> Receiver<N> {
    pub fn receive<F>(mut self, interrupted: &AtomicBool, f: F)
    where
        F: Fn(&Logline),
    {
        loop {
            let reference: usize = *self.reference_lock();
            while self.position != reference {
                if interrupted.load(ORDERING) {
                    return;
                }
                f(&self.next_log_line());
            }
            self.data.cond.wait(&mut self.reference_lock())
        }
    }

    fn reference_lock(&self) -> MutexGuard<usize> {
        self.data.reference.lock()
    }

    fn next_log_line(&mut self) -> MutexGuard<'_, Logline> {
        let ref_i = self.increment();
        self.data.data[ref_i].lock()
    }

    fn increment(&mut self) -> usize {
        self.position += 1;
        if self.position >= N {
            self.position = 0;
        }
        self.position
    }

    pub fn new(logdata: &Arc<LogData<N>>) -> Self {
        let position = *logdata.reference.lock();
        Self {
            position,
            data: Arc::clone(logdata),
        }
    }
}

#[cfg(test)]
mod test {
    use super::Receiver;
    use crate::logdata::LogData;
    use std::sync::{atomic::AtomicBool, Arc};

    #[test]
    fn test_receive() {
        let logdata = Arc::new(LogData::<5>::new());

        let receiver = Receiver::new(&logdata);

        logdata
            .receive(|buffer: &mut [u8]| {
                for (index, byte) in "Hello".as_bytes().iter().enumerate() {
                    buffer[index] = *byte;
                }
                if false {
                    Err(1)
                } else {
                    Ok("Hello".len())
                }
            })
            .unwrap();

        assert_eq!(
            format!("{}", "Hello"),
            format!("{}", logdata.data[1].lock())
        );

        let interrupted = AtomicBool::new(false);
        receiver.receive(&interrupted, |logline| {
            interrupted.store(true, std::sync::atomic::Ordering::SeqCst);
            logdata.cond.notify_one();
            assert_eq!(format!("{}", "Hello"), format!("{}", logline));
        })
    }
}
