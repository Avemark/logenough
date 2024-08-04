use crate::tcp::read_log_lines;
use eframe::egui;
use std::sync::mpsc;
use std::thread::JoinHandle;

pub struct LogWidgetState {
    logs: Vec<String>,
    title: String,
    worker: Option<JoinHandle<()>>,
}

impl LogWidgetState {
    fn new(title: String) -> Self {
        Self {
            logs: vec![],
            title,
            worker: None,
        }
    }

    fn show(&self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            for (index, line) in self.logs.iter().enumerate() {
                ui.push_id(index + 10, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(format!("{}", line));
                    });
                });
            }
        });
    }
}

impl std::ops::Drop for LogWidgetState {
    fn drop(&mut self) {
        let worker = self.worker.take();
        if let Some(handle) = worker {
            println!(
                "Trying to gently drop inner thread for widget {}",
                &self.title
            );
            handle.join().expect("couldn't join handle");
        }
    }
}

pub fn new_log_widget_worker(
    name: String,
    addr: String,
    on_done_tx: mpsc::SyncSender<()>,
    frame: egui::Context,
) -> (JoinHandle<()>, mpsc::SyncSender<egui::Context>) {
    let (show_tx, show_rc) = mpsc::sync_channel(0);
    let handle = std::thread::Builder::new()
        .name(format!("Log widget {}", &name))
        .spawn(move || {
            let mut state = LogWidgetState::new(name);
            let (logs_tx, logs_rc) = mpsc::sync_channel(1);
            let subthread_handle = std::thread::Builder::new()
                .name(format!("log widget '{}' background thread", &state.title))
                .spawn(move || {
                    read_log_lines(
                        Some(addr),
                        Some(|logline: String| {
                            println!("reading: {}", logline);
                            logs_tx
                                .send(logline)
                                .expect("could not send logline on channel");
                            frame.request_repaint();
                        }),
                    );
                })
                .expect("Could not spawn inner thread");
            state.worker = Some(subthread_handle);

            while let Ok(ctx) = show_rc.recv() {
                for line in logs_rc.try_iter() {
                    state.logs.push(line);
                }
                state.show(&ctx);
                let _ = on_done_tx.send(());
            }
        })
        .expect("Failed to spawn log widget thread");
    (handle, show_tx)
}
