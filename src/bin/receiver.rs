use std::sync::Mutex;

fn main() {
    //let buffer = Arc::new([EMPTY_BUFFER_LINE; 1024]);
}

const EMPTY_BUFFER_LINE: Mutex<[u8; 24]> = Mutex::new([0u8; 24]);
#[cfg(test)]
mod tests {
    use super::*;
    use std::net::UdpSocket;
    use std::sync::mpsc::channel;
    use std::sync::{Arc, Condvar};
    use std::thread;

    #[test]
    fn test_mutex_shit() {
        let buffer = Mutex::new([0u8; 24]);

        {
            let foo = buffer.lock().unwrap();
            println!("before: {:?}", foo);
        }
        {
            let mut foo = buffer.lock().unwrap();
            foo[0] = 2;
            foo[1] = 3;
        }
        {
            let foo = buffer.lock().unwrap();
            println!("after: {:?}", foo);
        }

        let bufferline = &buffer.lock().unwrap();
        assert_eq!(2, bufferline[0]);
        assert_eq!(3, bufferline[1]);
    }

    #[test]
    fn test_arc_shit() {
        let mutex = Arc::new(Mutex::new([0u8; 32]));

        let cloned = Arc::clone(&mutex);

        let udp = UdpSocket::bind("127.0.0.1:32471").unwrap();

        let keep = udp.try_clone().unwrap();

        let listener = thread::spawn(move || {
            udp.recv_from(&mut *mutex.lock().unwrap()).unwrap();
        });

        keep.send_to(&[4, 7, 1, 1], "127.0.0.1:32471").unwrap();

        listener.join().unwrap();

        assert_eq!(7, cloned.lock().unwrap()[1]);
    }

    #[test]
    fn test_condvar_stuff() {
        let condvar = Condvar::new();
    }
}
