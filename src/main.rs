use std::{
    env, fs,
    path::PathBuf,
    sync::mpsc::{self, Receiver, Sender, TryRecvError},
    thread,
};

use data::{AppData, ControlData, Message};

use eframe::egui;
use egui::Context;

mod data;
mod git;
mod ui;

fn main() -> Result<(), eframe::Error> {
    if env::var("PROFILING").is_ok() {
        puffin::set_scopes_on(true);
    }

    env_logger::init();

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        ..Default::default()
    };

    let path = get_initial_path();
    eframe::run_native(
        "Contrast",
        options,
        Box::new(|_cc| Box::new(MyApp::new(path))),
    )
}

fn get_initial_path() -> Option<PathBuf> {
    match env::args().nth(1) {
        Some(relative_path) => {
            let path = PathBuf::from(relative_path);
            match fs::canonicalize(path) {
                Ok(p) => Some(PathBuf::from(p)),
                Err(err) => {
                    eprintln!("Invalid path: {err}");
                    None
                }
            }
        }
        None => None,
    }
}

struct MyApp {
    app_data: Option<AppData>,
    control_data: ControlData,
    sender: Sender<Message>,
    receiver: Receiver<Message>,
}

impl MyApp {
    fn new(path: Option<PathBuf>) -> MyApp {
        let (sender, receiver) = mpsc::channel();

        if let Some(path) = path {
            sender
                .send(Message::LoadDiff(path))
                .expect("Channel closed unexpectedly!");
        }

        MyApp {
            app_data: None,
            control_data: ControlData::default(),
            sender,
            receiver,
        }
    }

    fn handle_messages(&mut self) {
        match self.receiver.try_recv() {
            Ok(msg) => match msg {
                Message::LoadDiff(path) => {
                    let s = self.sender.clone();
                    thread::spawn(move || match AppData::from_pathbuf(path) {
                        Ok(app_data) => s.send(Message::UpdateAppData(app_data)),
                        Err(_) => s.send(Message::ShowError("Error loading diff!".to_string())),
                    });
                }
                Message::UpdateAppData(app_data) => self.app_data = Some(app_data),
                Message::ChangeDiffType(diff_type) => self.control_data.diff_type = diff_type,
                Message::ChangeSelectedDiffIndex(i) => self.control_data.selected_diff_index = i,
                Message::ShowError(error) => {
                    self.control_data.error_information = error;
                    self.control_data.show_err_dialog = true;
                }
                Message::CloseError => {
                    self.control_data.error_information = "".to_string();
                    self.control_data.show_err_dialog = false;
                }
            },
            Err(err) => match err {
                TryRecvError::Disconnected => panic!("Channel closed unexpectedly!"),
                TryRecvError::Empty => (),
            },
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        puffin::profile_function!();
        puffin::GlobalProfiler::lock().new_frame();

        ui::show(ctx, &self.app_data, &self.control_data, &self.sender);

        self.handle_messages();
    }
}
