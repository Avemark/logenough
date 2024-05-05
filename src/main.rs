use ctrlc;
use logenough::logdata::LogData;
use logenough::logline::LockedLogline;
use logenough::receiver::Receiver;
use logenough::udp;
use parking_lot::Mutex;
use std::mem::size_of;
use std::net::UdpSocket;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

const LOG_LINE_COUNT: usize = 100000;

fn main() {
    let mem_size_buffer = 30_000;
    let data_size = size_of::<Mutex<LogData<LOG_LINE_COUNT>>>();
    let from_fn_multiplier = if cfg!(debug_assertions) { 10 } else { 2 };

    thread::Builder::new()
        .name("child thread".into())
        .stack_size(data_size * from_fn_multiplier + mem_size_buffer)
        .spawn(|| {
            let data = Arc::new(LogData::<LOG_LINE_COUNT>::new());
            let socket = UdpSocket::bind("127.0.0.1:4711").unwrap();
            let interrupted = Arc::new(AtomicBool::new(false));

            let listener_socket = socket.try_clone().unwrap();

            let interrupt = interrupted.clone();
            let handler_socket = socket.try_clone().expect("Failed to clone");
            let handler_data = Arc::clone(&data);
            ctrlc::set_handler(move || {
                println!("interrupting");
                interrupt.store(true, Ordering::SeqCst);
                handler_socket
                    .send_to("bye".as_bytes(), "127.0.0.1:4711")
                    .expect("Failed to send bye on udp socket");

                handler_data.cond.notify_all();
            })
            .expect("Could not set CTRL-C handler");

            thread::scope(|scope| {
                scope.spawn(|| {
                    println!("Started udp listener");
                    udp::listen(&data, &interrupted, listener_socket);
                });

                scope.spawn(|| {
                    Receiver::new(&data).receive(&interrupted, |logline| {
                        println!("f1 received something: '{}'", logline);
                    });
                });
            })
        })
        .expect("Failed to spawn child thread.")
        .join()
        .unwrap();
}
