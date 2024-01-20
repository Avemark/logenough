use circular_buffer::CircularBuffer;
use std::net::UdpSocket;
use std::sync::mpsc::{Receiver, Sender};

pub struct UdpListener<const N: usize> {
    tx: Sender<String>,
    qrx: Receiver<bool>,
    pub socket: UdpSocket,
    read_buffer: [u8; N],
}

pub const CANCEL: u8 = 24;
const EOT: u8 = 4;

impl<const N: usize> UdpListener<N> {
    fn next(&mut self) -> &[u8] {
        match self.socket.recv_from(&mut self.read_buffer) {
            Ok((bytes_recv, src)) => {
                // println!("Received {} bytes from {}", bytes_recv, src);
                &self.read_buffer[..bytes_recv]
            }
            Err(_error) => {
                // println!("problem: {}", error);
                &[EOT]
            }
        }
    }

    pub fn listen(&mut self) {
        loop {
            let bytes = self.next();
            if bytes == &[CANCEL] {
                if let Ok(message) = self.qrx.try_recv() {
                    if message {
                        break;
                    }
                }
            }
            if bytes == &[EOT] {
                break;
            }
            let str =
                String::from_utf8(bytes.to_owned()).expect("Failed to convert bytes to string");
            // println!("{}",str);
            self.lines.push_back(str);
            self.workers.retain(|worker| {
                match worker.send(
                    self.lines
                        .back()
                        .expect("Could not read what we just wrote")
                        .to_owned(),
                ) {
                    Ok(_) => true,
                    Err(_) => false, // This will not happen, we need another way to discard workers.
                }
            })
        }
    }

    pub fn new(qrx: Receiver<bool>, tx: Sender<String>, port: u16) -> Self {
        let socket = UdpSocket::bind(format!("127.0.0.1:{}", port)).unwrap();
        Self {
            qrx,
            tx,
            socket,
            read_buffer: [0u8; N],
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::channel;

    #[test]
    fn test_next() {
        let lines = CircularBuffer::<1024, String>::new();
        let (tx, _rx) = channel();
        let workers = vec![tx];
        let mut listener = UdpListener::new(lines, workers, 32471);

        listener
            .socket
            .send_to("A test message".as_bytes(), "127.0.0.1:32471")
            .expect("Could not send UDP test message");

        let next_message = String::from_utf8(listener.next().to_owned())
            .expect("Failed to convert bytes to string");
        assert_eq!("A test message", next_message)
    }

    #[test]
    fn test_listen() {
        let lines = CircularBuffer::<1024, String>::new();
        let (tx, rx) = channel();
        let workers = vec![tx];
        let mut listener = UdpListener::new(lines, workers, 32472);

        let test_messages = ["A test message", "A very very long message", "A short msg"];

        for message in test_messages {
            listener
                .socket
                .send_to(message.as_bytes(), "127.0.0.1:32472")
                .expect("Could not send UDP test message");
        }

        listener
            .socket
            .send_to(&[CANCEL], "127.0.0.1:32472")
            .expect("Failed to send CANCEL");

        listener.listen();

        let listener_ref = &listener;

        assert_eq!(3, listener_ref.lines.len());

        let mut messages = listener_ref.lines.iter();

        for message in test_messages {
            assert_eq!(
                &String::from(message),
                messages.next().unwrap_or(&String::from("Not that message"))
            );
            assert_eq!(
                String::from(message),
                rx.recv().unwrap_or(String::from("Not this message"))
            );
        }
    }
}
