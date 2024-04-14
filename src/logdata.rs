use crate::logline::LockedLogline;
use parking_lot::{Condvar, Mutex};
use std::array;

pub struct LogData<const N: usize> {
    pub data: [LockedLogline; N],
    pub reference: Mutex<usize>,
    pub cond: Condvar,
}

impl<const N: usize> LogData<N> {
    pub fn receive<F, E>(&self, f: F) -> Result<(), E>
    where
        F: Fn(&mut [u8]) -> Result<usize, E>,
    {
        self.increment();
        self.cond.notify_all();

        let reference = *self.reference.lock();
        println!("DATA: Taking a lock on data[{}]", reference);
        let mut logline = self.data[reference].lock();

        match f(&mut logline.buffer) {
            Ok(bytes_read) => {
                // println!("DATA: did read ref {} Ok", reference);
                logline.bytes_read = bytes_read;
                // println!("DATA: Notified");
                println!("DATA: releasing lock on data[{}]", reference);
                Ok(())
            }
            Err(error) => {
                logline.bytes_read = 0;
                Err(error)
            }
        }
    }

    fn increment(&self) -> usize {
        let mut position = self.reference.lock();
        *position += 1;
        if *position >= N {
            *position = 0;
        }
        // println!("DATA: incremented to {}", position);
        *position
    }

    pub fn new() -> Self {
        Self {
            data: array::from_fn(|_| LockedLogline::new()),
            reference: Mutex::new(N),
            cond: Condvar::new(),
        }
    }
}

impl<const N: usize> Default for LogData<N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::LogData;

    struct TErr {}

    #[test]
    fn test_increment_and_wrap() {
        let logdata = LogData::<2>::new();
        assert_eq!(0usize, *logdata.reference.lock());

        assert_eq!(1usize, logdata.increment());
        assert_eq!(1usize, *logdata.reference.lock());

        assert_eq!(0usize, logdata.increment());
    }

    #[test]
    fn test_receive_bytes_read() {
        let logdata = LogData::<2>::new();

        let receive = |_buffer: &mut [u8]| -> Result<usize, TErr> { Ok(1usize) };

        let result = logdata.receive(receive);

        assert!(result.is_ok());
        assert_eq!(1usize, logdata.data[1].lock().bytes_read);
    }

    #[test]
    fn test_receive_data() {
        let logdata = LogData::<2>::new();

        let receive = |buffer: &mut [u8]| -> Result<usize, TErr> {
            let hello = "Hello".as_bytes();
            for (index, byte) in hello.iter().enumerate() {
                buffer[index] = *byte;
            }
            Ok(hello.len())
        };

        let result = logdata.receive(receive);

        assert!(result.is_ok());
        assert_eq!(5usize, logdata.data[1].lock().bytes_read);
        assert_eq!(format!("Hello"), format!("{}", logdata.data[1].lock()));
    }
}
