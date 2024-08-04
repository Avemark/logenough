use crate::logline::LockedLogline;
use parking_lot::{Condvar, Mutex};
use std::sync::Arc;
use std::{array, thread};

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
        #[cfg(debug_assertions)]
        println!("DATA: Taking a lock on data[{}]", reference);
        let mut logline = self.data[reference].lock();

        match f(&mut logline.buffer) {
            Ok(bytes_read) => {
                logline.bytes_read = bytes_read;
                #[cfg(debug_assertions)]
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
        #[cfg(debug_assertions)]
        println!("DATA: incremented to {}", position);
        *position
    }

    pub fn new() -> Self {
        Self {
            data: array::from_fn(|_| LockedLogline::new()),
            reference: Mutex::new(N - 1),
            cond: Condvar::new(),
        }
    }

    pub fn build_logdata_in_sub_thread() -> Arc<LogData<N>> {
        let mem_size_buffer = 30_000;
        let data_size = size_of::<Mutex<LogData<N>>>();
        let from_fn_multiplier = if cfg!(debug_assertions) { 10 } else { 2 };

        thread::Builder::new()
            .name("child thread".into())
            .stack_size(data_size * from_fn_multiplier + mem_size_buffer)
            .spawn(|| Arc::new(LogData::<N>::new()))
            .unwrap()
            .join()
            .unwrap()
    }
}

impl<const N: usize> Default for LogData<N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::LogData;

    struct TErr {}

    #[test]
    fn test_increment_and_wrap() {
        let logdata = LogData::<2>::new();
        assert_eq!(1usize, *logdata.reference.lock());

        assert_eq!(0usize, logdata.increment());
        assert_eq!(0usize, *logdata.reference.lock());

        assert_eq!(1usize, logdata.increment());
    }

    #[test]
    fn test_receive_bytes_read() {
        let logdata = LogData::<2>::new();

        let receive = |_buffer: &mut [u8]| -> Result<usize, TErr> { Ok(1usize) };

        let result = logdata.receive(receive);

        assert!(result.is_ok());
        assert_eq!(1usize, logdata.data[0].lock().bytes_read);
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
        assert_eq!(5usize, logdata.data[0].lock().bytes_read);
        assert_eq!(format!("Hello"), format!("{}", logdata.data[0].lock()));
    }
}
