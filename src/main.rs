use std::{
    env, fs,
    path::PathBuf,
    sync::mpsc::{self, Receiver, Sender, TryRecvError},
    thread::{self},
};

use data::{AppData, ControlData, DiffType, Message};

use eframe::egui;
use egui::Context;
use notify::RecommendedWatcher;

mod data;
mod git;
mod ui;
mod watcher;

fn main() -> Result<(), eframe::Error> {
    let profiler = env::var("PROFILING").is_ok();
    if profiler {
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
        Box::new(move |_cc| Box::new(MyApp::new(path, profiler))),
    )
}

fn get_initial_path() -> Option<PathBuf> {
    env::args().nth(1).map(PathBuf::from).and_then(|p| {
        fs::canonicalize(p).map_or_else(
            |e| {
                eprintln!("Invalid path: {e}");
                None
            },
            Some,
        )
    })
}

struct MyApp {
    app_data: Option<AppData>,
    control_data: ControlData,
    sender: Sender<Message>,
    receiver: Receiver<Message>,
    watcher: Option<RecommendedWatcher>,
}

impl MyApp {
    fn new(path: Option<PathBuf>, profiler: bool) -> MyApp {
        let (sender, receiver) = mpsc::channel();

        if let Some(path) = path {
            load_repository(path, &sender);
        }

        MyApp {
            app_data: None,
            control_data: ControlData {
                profiler,
                ..Default::default()
            },
            sender,
            receiver,
            watcher: None,
        }
    }

    fn update_app_data(&mut self, app_data: &AppData) {
        match self.control_data.diff_type {
            DiffType::Modified => {
                if app_data.modified_diff_data.stats.files_changed == 0
                    && app_data.staged_diff_data.stats.files_changed != 0
                {
                    self.control_data.diff_type = DiffType::Staged
                }
            }
            DiffType::Staged => {
                if app_data.staged_diff_data.stats.files_changed == 0
                    && app_data.modified_diff_data.stats.files_changed != 0
                {
                    self.control_data.diff_type = DiffType::Modified
                }
            }
        }

        if self.watcher.is_none() {
            let p = app_data.project_path.clone();
            let should_refresh = self.control_data.should_refresh.clone();
            let sender = self.sender.clone();

            thread::spawn(move || watcher::run_watcher(PathBuf::from(p), should_refresh, sender));
        }
    }

    // only for messages that come from different threads
    fn handle_messages(&mut self) {
        match self.receiver.try_recv() {
            Ok(msg) => match msg {
                Message::UpdateAppData(app_data) => {
                    self.update_app_data(&app_data);
                    self.app_data = Some(app_data);
                }
                Message::UpdateWatcher(watcher) => self.watcher = Some(watcher),
                Message::ShowError(error) => {
                    self.control_data.error_information = error;
                    self.control_data.show_err_dialog = true;
                }
            },
            Err(err) => match err {
                TryRecvError::Disconnected => panic!("Channel closed unexpectedly!"),
                TryRecvError::Empty => (),
            },
        }

        let mutex_guard = self.control_data.should_refresh.lock();
        if mutex_guard.is_err() {
            self.sender
                .send(Message::ShowError("Error refreshing diff!".to_string()))
                .expect("Channel closed unexpectedly!");
            return;
        }

        // acceptable unwrap() since result is checked beforehand
        let mut should_refresh = mutex_guard.unwrap();

        if *should_refresh {
            if let Some(app_data) = &self.app_data {
                load_repository(PathBuf::from(app_data.project_path.clone()), &self.sender);
            }
            *should_refresh = false;
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        puffin::profile_function!();
        puffin::GlobalProfiler::lock().new_frame();

        if let Some(app_data) = &mut self.app_data {
            egui::SidePanel::right("git log panel")
                .max_width(frame.info().window_info.size.x / 2.0)
                .show_animated(ctx, self.control_data.log_open, |ui| {
                    ui::log::ui(ui, &app_data.commits, &mut self.control_data);
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui::selection(ui, ctx, &mut self.control_data, &self.sender);
            if let Some(app_data) = &mut self.app_data {
                ui::main(ui, app_data, &mut self.control_data);
            }
        });

        if self.control_data.profiler {
            self.control_data.profiler = puffin_egui::profiler_window(ctx);
        }

        self.handle_messages();
    }
}

fn load_repository(path: PathBuf, sender: &Sender<Message>) {
    let s = sender.clone();
    thread::spawn(move || match AppData::from_pathbuf(path) {
        Ok(app_data) => s
            .send(Message::UpdateAppData(app_data))
            .expect("Channel closed unexpectedly!"),
        Err(_) => s
            .send(Message::ShowError("Error loading diff!".to_string()))
            .expect("Channel closed unexpectedly!"),
    });
}
