use std::net::UdpSocket;
use ctrlc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::mpsc::channel;
use std::thread;
use circular_buffer::CircularBuffer;
use logenough::listener::Listener;

fn main() {
    let interrupted = Arc::new(AtomicBool::new(true));

    let got_int = interrupted.clone();
    ctrlc::set_handler(move || {
        got_int.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl+C handler");

    let listen_socket = UdpSocket::bind("127.0.0.1:32471").expect("Could not bind to socket");
    let socket_ref = &listen_socket;

    socket_ref.send("Hello".as_bytes()).expect("Failed to send on socket");

    let lines = CircularBuffer::<1024, String>::new();

    let senders: Vec<Sender<String>> = vec!([]);
    let

    let listener = Listener::new(lines, )

    let listener_thread = thread::spawn(move || {

    })
}
