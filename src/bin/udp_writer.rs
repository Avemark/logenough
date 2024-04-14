use std::net::UdpSocket;

fn main() {
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();

    socket
        .send_to("This is a message".as_bytes(), "127.0.0.1:4711")
        .unwrap();
}
