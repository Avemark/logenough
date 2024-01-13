use crate::worker::Worker;
use circular_buffer::CircularBuffer;
use std::net::UdpSocket;

pub struct Listener<const N: usize> {
    lines: CircularBuffer<N, String>,
    workers: Box<[Worker]>,
    socket: UdpSocket,
    read_buffer: [u8; 1024],
}

impl<const N: usize> Listener<N> {
    fn next(&mut self) -> String {
        match self.socket.recv_from(&mut self.read_buffer) {
            Ok((bytes_recv, src)) => {
                println!("Received {} bytes from {}", bytes_recv, src);
                String::from_utf8(self.read_buffer[..bytes_recv].to_owned())
                    .expect("Failed to convert bytes to string")
            }
            Err(_) => String::from("Failure"),
        }
    }

    pub fn listen(&mut self) {
        loop {
            let str = self.next();
            if str == String::from("Exit") {
                break
            }
            self.lines.push_back(str);
        }
    }

    pub fn new(lines: CircularBuffer<N, String>, workers: Box<[Worker]>, port: u8) -> Self {
        let socket = UdpSocket::bind(format!("127.0.0.1:{}", port)).unwrap();
        Self {
            lines,
            workers,
            socket,
            read_buffer: [0u8; 1024],
        }
    }
}
