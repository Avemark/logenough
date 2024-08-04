use logenough::logdata::LogData;
use logenough::{set_ctrl_c_handler, tcp, udp};
use std::net::UdpSocket;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread;

const LOG_LINE_COUNT: usize = 2_000;

fn main() {
    let logdata = LogData::<LOG_LINE_COUNT>::build_logdata_in_sub_thread();

    let socket = UdpSocket::bind("127.0.0.1:4711").unwrap();
    let interrupted = Arc::new(AtomicBool::new(false));

    let listener_socket = socket.try_clone().unwrap();

    let interrupt = interrupted.clone();
    let handler_socket = socket.try_clone().expect("Failed to clone");
    let handler_data = Arc::clone(&logdata);

    set_ctrl_c_handler(interrupt, handler_socket, handler_data);

    thread::scope(|scope| {
        scope.spawn(|| {
            println!("Started udp listener");
            udp::listen(&logdata, &interrupted, listener_socket);
        });

        scope.spawn(|| {
            println!("Starting Tcp Listener");
            tcp::accept_connections(&logdata, &interrupted)
        });
    });
}
