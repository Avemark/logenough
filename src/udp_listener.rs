use crate::log_data::Logdata;
use std::net::UdpSocket;
use std::sync::atomic::{AtomicBool, Ordering};
// use std::sync::Arc;
//
// pub fn listen<const N: usize, const M: usize>(
//     data: Arc<[Mutex<[u8; M]>; N]>,
//     interrupted: &AtomicBool,
//     socket: UdpSocket,
//     reference: Arc<Reference<N>>,
// ) {
//     while !interrupted.load(Ordering::Relaxed) {
//         socket
//             .recv_from(&mut *data[reference.increment()].lock().unwrap())
//             .expect("Could not listen on UDP socket");
//         reference.notify();
//     }
// }

#[allow(dead_code)]
pub fn listen<const N: usize>(
    log: Logdata<N>,
    interrupted: &AtomicBool,
    socket: UdpSocket,
) -> Logdata<N> {
    while !interrupted.load(Ordering::SeqCst) {
        let reference = log.reference.increment();
        socket
            .recv_from(&mut *log.data[reference].lock().unwrap())
            .expect("Could not listen on UDP socket");

        log.reference.notify();
    }
    log
}

#[cfg(test)]
mod tests {
    use crate::log_data::Logdata;
    use crate::udp_listener::listen;
    use std::net::UdpSocket;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::thread;
    use std::thread::sleep;
    use std::time::Duration;

    const TEST_ADDRESS: &str = "127.0.0.1:32471";
    #[test]
    fn test_listen() {
        let log = Logdata::<5>::new();
        let log_clone = log.clone();
        let interrupt = AtomicBool::new(false);
        let socket = UdpSocket::bind(TEST_ADDRESS).unwrap();

        thread::scope(|s| {
            let listener = s.spawn(|| listen(log, &interrupt, socket.try_clone().unwrap()));
            sleep(Duration::new(0, 1000000));
            socket.send_to("Hello".as_bytes(), TEST_ADDRESS).unwrap();

            interrupt.store(true, Ordering::SeqCst);
            socket.send_to("quit".as_bytes(), TEST_ADDRESS).unwrap();
            listener.join().unwrap();
        });

        let received_data = &*log_clone.data[1].lock().unwrap();
        let str_len = received_data.iter().position(|b| b == &0u8).unwrap();
        assert_eq!(
            String::from_utf8(received_data[0..str_len].into()).unwrap(),
            String::from("Hello")
        );
    }
}
