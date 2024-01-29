use crate::reference::Reference;
use std::net::UdpSocket;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::{Arc, Mutex};

pub mod listener;
mod reference;

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
