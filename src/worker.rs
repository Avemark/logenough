use std::fmt::Display;
use std::sync::mpsc::{channel, Receiver, Sender};

pub struct Worker<T>
{
    rx: Receiver<T>,
    number: u8
}

impl<T> Worker<T>
where
    T: Display
{
    pub fn listen(&self) {
        while let Ok(msg) = self.rx.recv() {

            println!("Worker {}: {}", self.number, msg);
        }
    }

    pub fn new(number: u8) -> (Sender<T>, Self) {
        let (tx, rx) = channel();
        let instance = Self { rx, number };

        (tx, instance)
    }
}
