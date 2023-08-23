use std::{
    env, fs,
    path::{Path, PathBuf},
    sync::{
        mpsc::{self, Receiver, Sender, TryRecvError},
        Arc, Mutex,
    },
    thread::{self},
};

use data::{AppData, ControlData, DiffType, Message};

use eframe::egui;
use egui::Context;
use ignore::{gitignore::Gitignore, Error};
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};

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
    }

    fn handle_messages(&mut self) {
        match self.receiver.try_recv() {
            Ok(msg) => match msg {
                Message::LoadDiff(path) => {
                    let s = self.sender.clone();
                    thread::spawn(move || match AppData::from_pathbuf(path) {
                        Ok(app_data) => s
                            .send(Message::UpdateAppData(app_data))
                            .expect("Channel closed unexpectedly!"),
                        Err(_) => s
                            .send(Message::ShowError("Error loading diff!".to_string()))
                            .expect("Channel closed unexpectedly!"),
                    });
                }
                Message::UpdateAppData(app_data) => {
                    self.update_app_data(&app_data);

                    if self.watcher.is_none() {
                        let p = app_data.project_path.clone();
                        let should_refresh = self.control_data.should_refresh.clone();
                        let sender = self.sender.clone();

                        thread::spawn(move || {
                            run_watcher(PathBuf::from(p), should_refresh, sender)
                        });
                    }

                    self.app_data = Some(app_data);
                }
                Message::UpdateWatcher(watcher) => self.watcher = Some(watcher),
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

        let mut should_refresh = self.control_data.should_refresh.lock().unwrap();

        if *should_refresh {
            if let Some(app_data) = &self.app_data {
                self.sender
                    .send(Message::LoadDiff(PathBuf::from(
                        app_data.project_path.clone(),
                    )))
                    .expect("Channel closed unexpectedly!");
            }
            *should_refresh = false;
        }
    }
}

fn get_gitignore(path: &Path) -> Result<Gitignore, Error> {
    puffin::profile_function!();
    let (gitignore, error) = Gitignore::new(path.join(".gitignore"));
    if let Some(e) = error {
        return Err(e);
    }
    Ok(gitignore)
}

struct WatcherError;

// TODO: error handling
fn run_watcher(
    path: PathBuf,
    should_refresh: Arc<Mutex<bool>>,
    sender: Sender<Message>,
) -> Result<(), WatcherError> {
    puffin::profile_function!();

    let gitignore = get_gitignore(&path).map_err(|_| WatcherError {})?;

    let s = sender.clone();
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| match res {
        Ok(event) => {
            for p in &event.paths {
                if !gitignore.matched_path_or_any_parents(p, false).is_ignore() {
                    match should_refresh.lock() {
                        Ok(mut sr) => *sr = true,
                        Err(_) => {
                            s.send(Message::ShowError("Error acquiring mutex".to_string()))
                                .expect("");
                        }
                    }
                }
            }
        }
        Err(e) => println!("watch error: {:?}", e),
    })
    .map_err(|_| WatcherError {})?;

    watcher
        .watch(&path, RecursiveMode::Recursive)
        .map_err(|_| WatcherError {})?;

    sender
        .send(Message::UpdateWatcher(watcher))
        .expect("Channel closed unexpectedly!");

    Ok(())
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        puffin::profile_function!();
        puffin::GlobalProfiler::lock().new_frame();

        ui::show(ctx, &self.app_data, &self.control_data, &self.sender);

        self.handle_messages();
    }
}
