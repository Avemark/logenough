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
    cond: Condvar,
    reference: AtomicUsize,
}

impl<const N: usize> Reference<N> {
    fn increment(&self) -> usize {
        self.reference.fetch_add(1, AcqRel);
        self.reference
            .compare_exchange(N, 0usize, AcqRel, Relaxed)
            .unwrap_or_else(|num| num)
    }

    fn notify(&self) {
        self.cond.notify_all();
    }
}
