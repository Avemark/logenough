use std::net::UdpSocket;
use std::sync::atomic::Ordering::{AcqRel, Relaxed, SeqCst};
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::{Arc, Condvar, Mutex};

pub mod listener;

pub fn listen<const N: usize, const M: usize>(
    data: Arc<[Mutex<[u8; M]>; N]>,
    interrupted: &AtomicBool,
    socket: UdpSocket,
    reference: Arc<Reference<N>>,
) {
    while !interrupted.load(Relaxed) {
        socket
            .recv_from(&mut *data[reference.increment()].lock().unwrap())
            .expect("Could not listen on UDP socket");
        reference.notify();
    }
}

pub struct Reference<const N: usize> {
    pub cond: Condvar,
    pub reference: Mutex<usize>,
}

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
