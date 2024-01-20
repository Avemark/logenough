use circular_buffer::CircularBuffer;
use crossbeam_channel::bounded;
use ctrlc;
use logenough::listener::{UdpListener, CANCEL};
use std::sync::mpsc::Sender;
use std::thread;

fn main() {
    //    let lines = CircularBuffer::<1024, String>::new();

    let (qtx, qrx) = bounded(1);

    let mut udp_listener = UdpListener::new(lines, senders, 32471);

    let socket = udp_listener
        .socket
        .try_clone()
        .expect("Could not clone UDP socket");

    ctrlc::set_handler(move || {
        qtx.send(true);
        socket.send(&[CANCEL]);
    })
    .expect("Error setting Ctrl+C handler");

    let listener_thread = thread::spawn(move || {
        udp_listener.listen();
    });
}
