use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::JoinHandle;

pub struct Worker<'a>
{
    rx: Receiver<&'a [u8]>,
    tx: Sender<&'a [u8]>,
    stream: TcpStream
}

impl<'a> Worker<'a>
{
    pub fn listen(&mut self) {
        while let Ok(msg) = self.rx.recv() {
            self.stream.write(&msg).expect("Failed to write to TCP stream");
            println!("Worker: {}", String::from_utf8(msg.into()).expect("Could not serialize string from input"));
        }
    }

    pub fn sender(&self) -> Sender<&'a [u8]> {
        self.tx.clone()
    }

    pub fn collect_streams(tx: Sender<TcpStream>, abort: Receiver<bool>, port: u16) -> (JoinHandle<()>, TcpListener) {
        todo!()
    }

    pub fn spawn(port: u16) -> Self {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).expect("Could not bind to tcp port");
        let (stream, addr) = listener.accept().expect("Accept failure");
        println!("Accepted connection: {}", addr);
        let (tx, rx) = channel::<TcpStream>();
        Self::new(stream)
    }

    pub fn new(stream: TcpStream) -> Self {
        let (tx, rx) = channel();
        Self { rx, tx, stream }
    }
}
