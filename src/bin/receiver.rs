use std::sync::Mutex;

fn main() {
    //let buffer = Arc::new([EMPTY_BUFFER_LINE; 1024]);
}

const EMPTY_BUFFER_LINE: Mutex<[u8; 24]> = Mutex::new([0u8; 24]);
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::channel;
    use std::sync::Arc;

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
        let mut simple = Arc::new(13);

        let foo = Arc::get_mut(&mut simple).unwrap();

        *foo = 8;

        assert_eq!(8, *simple);

        let mut mutex = Arc::new(Mutex::new([1, 2, 3]));

        let cloned = Arc::clone(&mutex);
        {
            cloned.lock().unwrap()[1] = 5;
        }

        assert_eq!(5, mutex.lock().unwrap()[1]);
    }
}
