use eframe::egui::Context;
use eframe::{egui, Frame};
use logenough::logdata::LogData;
use logenough::{set_ctrl_c_handler, udp};
use std::net::UdpSocket;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread;

const LOG_LINE_COUNT: usize = 2000;

fn main() {
    let eframe_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    let logdata = LogData::<LOG_LINE_COUNT>::build_logdata_in_sub_thread();
    let (tx, rx): (Sender<String>, Receiver<String>) = channel();

    let socket = UdpSocket::bind("127.0.0.1:4711").unwrap();
    let interrupted = Arc::new(AtomicBool::new(false));

    let listener_socket = socket.try_clone().unwrap();

    let interrupt = interrupted.clone();
    let handler_socket = socket.try_clone().expect("Failed to clone");
    let handler_data = Arc::clone(&logdata);

    set_ctrl_c_handler(interrupt, handler_socket, handler_data);
    thread::scope(|scope| {
        scope.spawn(|| {
            println!("Started udp listener");
            udp::listen(&logdata, &interrupted, listener_socket);
        });

        scope.spawn(|| {
            logenough::receiver::Receiver::new(&logdata).receive(&interrupted, |logline| {
                tx.send(format!("{}", logline))
                    .expect("Failed to send logline on channel");
            });
        });
        eframe::run_native(
            "Logs",
            eframe_options,
            Box::new(|_cc| Ok(Box::<Logwindow>::new(Logwindow::new(rx)))),
        )
        .expect("Eframe not happy");
    });
}

struct Logwindow {
    logs: Vec<String>,
    incoming: Receiver<String>,
}

impl Logwindow {
    fn new(receiver: Receiver<String>) -> Self {
        Self {
            logs: vec![],
            incoming: receiver,
        }
    }
}

impl eframe::App for Logwindow {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        for line in self.incoming.try_iter() {
            self.logs.push(line);
            if self.logs.len() > 20 {
                self.logs.remove(0);
            }
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            for line in &self.logs {
                ui.horizontal(|ui| {
                    ui.label(line);
                });
            }
        });
    }
}
