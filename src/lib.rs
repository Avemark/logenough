#![allow(dead_code)]

pub mod logdata;
pub mod logline;
pub mod receiver;
pub mod udp;

#[cfg(test)]
mod test {
    use crate::logdata::LogData;
    use crate::receiver::Receiver;
    use crate::udp;
    use std::sync::Arc;
    use std::thread;
    use std::{net::UdpSocket, sync::atomic::AtomicBool};

    #[test]
    fn test_consuming() {
        let data = Arc::new(LogData::<5>::new());
        let socket = UdpSocket::bind("127.0.0.1:4711").unwrap();
        let interrupted = AtomicBool::new(false);

        let listener_socket = socket.try_clone().unwrap();
        let listener_data = Arc::clone(&data);

        thread::scope(|scope| {
            scope.spawn(|| {
                udp::listen(listener_data, &interrupted, listener_socket);
            });

            scope.spawn(|| {
                // let receiver = Receiver::new(&data);
                //receiver.receive(&interrupted, |logline| {
                //   assert_eq!("Hello", format!("{}", logline))
                //});
            });

            interrupted.store(true, std::sync::atomic::Ordering::SeqCst);

            socket
                .send_to("Hello".as_bytes(), "127.0.0.1:4711")
                .unwrap();

            data.cond.notify_all();

            socket.send_to("Quit".as_bytes(), "127.0.0.1:4711").unwrap();
        })
    }
}
