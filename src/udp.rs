use std::{net::UdpSocket, sync::atomic::AtomicBool};

use crate::logdata::LogData;

pub fn listen<const N: usize>(data: LogData<N>, interrupted: &AtomicBool, socket: UdpSocket) {
    while !interrupted.load(std::sync::atomic::Ordering::Relaxed) {
        data.receive(|buf: &mut [u8]| match socket.recv_from(buf) {
            Ok((bytes_received, _addr)) => Ok(bytes_received),
            Err(e) => Err(e),
        })
        .expect("Could not listen to udp socket?")
    }
}
