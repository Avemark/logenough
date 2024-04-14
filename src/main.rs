use ctrlc;
use logenough::logdata::LogData;
use logenough::logline::LockedLogline;
use logenough::receiver::Receiver;
use logenough::udp;
use std::mem::size_of;
use std::net::UdpSocket;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::thread::sleep;
use std::time::Duration;

const LOG_LINE_COUNT: usize = 2000;

fn main() {
    println!("per line: {}", size_of::<LockedLogline>());
    println!(
        "expected total: {}",
        size_of::<LockedLogline>() * LOG_LINE_COUNT
    );
    println!(
        "size of the whole thing? {}",
        size_of::<LogData<LOG_LINE_COUNT>>()
    );
    let child = thread::Builder::new()
        .name("child thread".into())
        .stack_size(10000000)
        .spawn(|| {
            let data = Arc::new(LogData::<LOG_LINE_COUNT>::new());
            let socket = UdpSocket::bind("127.0.0.1:4711").unwrap();
            let interrupted = Arc::new(AtomicBool::new(false));

            let listener_socket = socket.try_clone().unwrap();
            let listener_data = Arc::clone(&data);

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
                    udp::listen(listener_data, &interrupted, listener_socket);
                });

                scope.spawn(|| {
                    Receiver::new(&data).receive(&interrupted, |logline| {
                        println!("f() received something: '{}'", logline);
                    });
                });
            })
        });
    child.unwrap().join().unwrap();
}
