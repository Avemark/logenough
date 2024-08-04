use crate::logdata::LogData;
use crate::logline::Logline;
use crate::receiver::Receiver;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

struct TcpError;

pub fn accept_connections<const N: usize>(
    data: &Arc<LogData<N>>,
    interrupted: &AtomicBool,
) -> Result<(), std::io::Error> {
    thread::scope(|scope| {
        let tcp_listener =
            TcpListener::bind("127.0.0.1:91").expect("Failed to bind to 127.0.0.1:91");
        let mut streams: Vec<TcpStream> = vec![];
        while !interrupted.load(SeqCst) {
            let (mut stream, addr) = tcp_listener.accept().expect("Tcp stream accept failure?");
            let mut line = Logline {
                bytes_read: 0usize,
                buffer: [0u8; 508],
            };
            line.bytes_read = stream
                .read(&mut line.buffer)
                .expect("No greeting sent by tcp client!");
            match format!("{}", line).as_str() {
                "hello" => {
                    println!("Received hello, opening connection")
                }
                "bye" => {
                    println!("Received bye, exiting");
                    continue;
                }
                _ => {
                    println!("unexpected message!: {}", line)
                }
            }
            println!("Accepting connection from {}", addr);

            streams.push(stream.try_clone().expect("Couldn't clone stream"));

            scope.spawn(move || {
                let receiver = Receiver::new(&data);
                receiver.receive(&interrupted, |logline| {
                    let mut tcp_stream = stream
                        .try_clone()
                        .expect("could not make new stream for the receiver");
                    tcp_stream
                        .write(
                            format!(
                                "{pad}{len}:{buf},",
                                pad = left_pad(logline.bytes_read),
                                len = logline.bytes_read,
                                buf = logline
                            )
                            .as_bytes(),
                        )
                        .expect("Failed to send data over TCP stream");
                });
            });
        }

        println!("interrupted, quitting");

        for stream in streams {
            stream.shutdown(Shutdown::Both).err().or_else(|| {
                println!("stream shutdown error");
                None
            });
        }
    });
    Ok(())
}

fn left_pad<'a>(integer: usize) -> &'a str {
    if integer > 99 {
        ""
    } else if integer > 9 {
        "0"
    } else {
        "00"
    }
}

pub fn read_log_lines<F>(addr: Option<String>, mut on_receive: Option<F>)
where
    F: FnMut(String),
{
    let addr = addr.unwrap_or("127.0.0.1:91".into());
    let mut stream = TcpStream::connect(addr).expect("Connection failed");

    println!("Connected!");

    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .expect("Could not set write timeout");

    stream
        .write("hello".as_bytes())
        .expect("Failed to write hello");

    stream
        .set_read_timeout(Some(Duration::from_secs(60)))
        .expect("failed to set read timeout?");

    loop {
        match read_header(&mut stream) {
            Ok(expected_length) => {
                match read_message(&mut stream, expected_length) {
                    Ok(message) => {
                        if let Some(callback) = &mut on_receive {
                            callback(message);
                        }
                    }
                    Err(_) => break,
                };
            }
            Err(_) => break,
        };
    }
}

fn read_message(stream: &mut TcpStream, expected_length: usize) -> Result<String, std::io::Error> {
    let mut log_buf = [0u8; 509];
    match stream.read_exact(&mut log_buf[..expected_length + 1]) {
        Ok(_) => Ok(String::from_utf8_lossy(&log_buf[..expected_length]).into()),
        Err(e) => Err(e),
    }
}

fn read_header(stream: &mut TcpStream) -> Result<usize, std::io::Error> {
    let mut len_buf = [0u8; 4];

    match stream.read_exact(&mut len_buf) {
        Ok(()) => {
            let str = String::from_utf8_lossy(&len_buf[..3]);
            Ok(usize::from_str_radix(&str, 10)
                .expect(&format!("Failed to parse {} into usize", &str)))
        }
        Err(e) => Err(e),
    }
}
