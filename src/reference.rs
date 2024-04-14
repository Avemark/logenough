use std::sync::{Condvar, Mutex};

#[derive(Debug)]
pub struct Reference<const N: usize> {
    pub cond: Condvar,
    pub reference: Mutex<usize>,
}
#[allow(dead_code)]
impl<const N: usize> Reference<N> {
    pub fn increment(&self) -> usize {
        let mut reference = self.reference.lock().unwrap();
        *reference = reference.wrapping_add(1);
        if *reference >= N {
            *reference = 0usize;
        }
        *reference
    }

    pub fn notify(&self) {
        self.cond.notify_all();
    }

    pub fn new(num: usize) -> Self {
        Self {
            cond: Condvar::new(),
            reference: Mutex::new(num),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::reference::Reference;

    #[test]
    fn test_wrap() {
        let ref_50 = Reference::<50>::new(48);

        ref_50.increment();
        assert_eq!(49, *ref_50.reference.lock().unwrap());
        ref_50.increment();
        assert_eq!(0, *ref_50.reference.lock().unwrap());
    }
}
