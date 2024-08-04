#![allow(dead_code)]

use crate::logdata::LogData;
use std::io::Write;
use std::net::{TcpStream, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub mod logdata;
pub mod logline;
pub mod receiver;
pub mod tcp;
pub mod udp;

pub mod log_widget;

pub fn set_ctrl_c_handler<const N: usize>(
    interrupt: Arc<AtomicBool>,
    handler_socket: UdpSocket,
    handler_data: Arc<LogData<N>>,
) {
    ctrlc::set_handler(move || {
        println!("interrupting");
        interrupt.store(true, Ordering::SeqCst);
        handler_socket
            .send_to("bye".as_bytes(), "127.0.0.1:4711")
            .expect("Failed to send bye on udp socket");

        match TcpStream::connect("127.0.0.1:91") {
            Ok(mut stream) => {
                stream
                    .write("bye".as_bytes())
                    .expect("Bye message delivery failed");
            }
            Err(e) => {
                println!(
                    "Failed to connect to tcp listener for goodbye message. {}",
                    e
                );
            }
        }

        handler_data.cond.notify_all();
    })
    .expect("Could not set CTRL-C handler");
}

#[cfg(test)]
mod lib_test {
    use crate::logdata::LogData;
    use crate::receiver::Receiver;
    use crate::udp;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;
    use std::{net::UdpSocket, sync::atomic::AtomicBool};

    #[test]
    fn test_consuming() {
        let data = Arc::new(LogData::<5>::new());
        let socket = UdpSocket::bind("127.0.0.1:4711").unwrap();
        let interrupted = AtomicBool::new(false);

        let listener_socket = socket.try_clone().unwrap();

        thread::scope(|scope| {
            scope.spawn(|| {
                udp::listen(&data, &interrupted, listener_socket);
                println!("Exiting udp listen thread");
            });

            scope.spawn(|| {
                let receiver = Receiver::new(&data);
                receiver.receive(&interrupted, |logline| {
                    println!("receiving thing!");
                    assert_eq!("Hello", format!("{}", logline))
                });
                println!("exiting receiver thread");
            });

            thread::sleep(Duration::from_millis(50));

            socket
                .send_to("Hello".as_bytes(), "127.0.0.1:4711")
                .unwrap();

            interrupted.store(true, std::sync::atomic::Ordering::Release);

            data.cond.notify_all();

            socket.send_to("Quit".as_bytes(), "127.0.0.1:4711").unwrap();
        });
    }
}
