use eframe::egui::Context;
use eframe::{egui, Frame};
use logenough::log_widget::new_log_widget_worker;
use std::sync::mpsc;
use std::sync::mpsc::SyncSender;
use std::thread::JoinHandle;

fn main() {
    let eframe_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Logs",
        eframe_options,
        Box::new(|_cc| Ok(Box::<Logwindow>::new(Logwindow::new()))),
    )
    .expect("Eframe not happy");
}

struct MainWindowState {
    connection_name: String,
    connection_addr: String,
}

impl MainWindowState {
    fn new() -> Self {
        Self {
            connection_name: "".to_string(),
            connection_addr: "".to_string(),
        }
    }
}

struct Logwindow {
    widgets: Vec<(JoinHandle<()>, SyncSender<egui::Context>)>,
    on_done_rc: mpsc::Receiver<()>,
    on_done_tx: mpsc::SyncSender<()>,
    main_window_state: MainWindowState,
}

impl Logwindow {
    fn new() -> Self {
        let (on_done_tx, on_done_rc) = mpsc::sync_channel(0);
        Self {
            widgets: vec![],
            on_done_rc,
            on_done_tx,
            main_window_state: MainWindowState::new(),
        }
    }

    fn connect(&mut self, name: String, addr: String, frame: egui::Context) {
        self.widgets.push(new_log_widget_worker(
            name,
            addr,
            self.on_done_tx.clone(),
            frame,
        ));
    }
}

impl eframe::App for Logwindow {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.push_id(1, |ui| {
                ui.horizontal(|ui| {
                    let name_label = ui.label("Connection name:");
                    ui.text_edit_singleline(&mut self.main_window_state.connection_name)
                        .labelled_by(name_label.id);
                });
                ui.horizontal(|ui| {
                    let addr_label = ui.label("Connection addr:");
                    ui.text_edit_singleline(&mut self.main_window_state.connection_addr)
                        .labelled_by(addr_label.id);
                });
                if ui.button("Connect!").clicked() {
                    self.connect(
                        self.main_window_state.connection_name.clone(),
                        self.main_window_state.connection_addr.clone(),
                        ctx.clone(),
                    )
                }
            });
            for (_, show_tx) in &self.widgets {
                let _ = show_tx.send(ctx.clone());
            }

            for _ in 0..self.widgets.len() {
                let _ = self.on_done_rc.recv();
            }
        });
    }
}
