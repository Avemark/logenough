use circular_buffer::CircularBuffer;
use std::net::UdpSocket;
use std::sync::mpsc::Sender;

pub struct Listener<const N: usize> {
    lines: CircularBuffer<N, String>,
    workers: Vec<Sender<String>>,
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
            println!("{}",str);
            self.lines.push_back(str);
            self.workers.retain(|worker| {
                match worker.send(self.lines.back().expect("Could not read what we just wrote").to_owned()) {
                    Ok(_) => true,
                    Err(_) => false // This will not happen, we need another way to discard workers.
                }
            })
        }
    }

    pub fn new(lines: CircularBuffer<N, String>, workers: Vec<Sender<String>>, port: u16) -> Self {
        let socket = UdpSocket::bind(format!("127.0.0.1:{}", port)).unwrap();
        Self {
            lines,
            workers,
            socket,
            read_buffer: [0u8; 1024],
        }
    }
}
#[cfg(test)]
mod tests {
    use std::sync::mpsc::channel;
    use super::*;

    #[test]
    fn test_next() {
        let lines = CircularBuffer::<1024, String>::new();
        let (tx, _rx) = channel();
        let workers = vec![tx];
        let mut listener = Listener::new(lines, workers, 32471);

        listener.socket.send_to("A test message".as_bytes(), "127.0.0.1:32471").expect("Could not send UDP test message");

        assert_eq!("A test message", listener.next())
    }

    #[test]
    fn test_listen() {
        let lines = CircularBuffer::<1024, String>::new();
        let (tx, rx) = channel();
        let workers = vec![tx];
        let mut listener = Listener::new(lines, workers, 32471);

        listener.socket.send_to("A test message".as_bytes(), "127.0.0.1:32471").expect("Could not send UDP test message");

        listener.socket.send_to("A very very long message".as_bytes(), "127.0.0.1:32471").expect("Could not send UDP test message");

        listener.socket.send_to("A short msg".as_bytes(), "127.0.0.1:32471").expect("Could not send UDP test message");
        listener.socket.send_to("Exit".as_bytes(), "127.0.0.1:32471").expect("Could not send UDP test message");

        listener.listen();

        let listener_ref = &listener;

        assert_eq!(3, listener_ref.lines.len());
        let mut messages = listener_ref.lines.iter();

        assert_eq!(&String::from("A test message"), messages.next().unwrap_or(&String::from("Not that message")));
        assert_eq!(&String::from("A very very long message"), messages.next().unwrap_or(&String::from("Not that message")));
        assert_eq!(&String::from("A short msg"), messages.next().unwrap_or(&String::from("Not that message")));

        assert_eq!(String::from("A test message"), rx.recv().unwrap_or(String::from("Not this message")));
    }
}
