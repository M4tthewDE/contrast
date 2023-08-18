use egui::{Align, Layout};
use std::path::PathBuf;
use ui::{error_dialog, DiffAreaWidget, FilesAreaWidget, ProjectAreaWidget, SelectionAreaWidget};

use git::{Diff, DiffParsingError, Stats};

use eframe::egui;

mod git;
mod ui;

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        ..Default::default()
    };

    eframe::run_native("Contrast", options, Box::new(|_cc| Box::<MyApp>::default()))
}

#[derive(Default)]
struct MyApp {
    app_data: Option<AppData>,
    control_data: ControlData,
}

#[derive(Default)]
pub struct ControlData {
    show_err_dialog: bool,
    error_information: String,
}

#[derive(Clone)]
pub struct AppData {
    project_path: String,
    diffs: Vec<Diff>,
    stats: Stats,
    selected_diff_index: usize,
}

enum AppDataCreationError {
    Parsing,
}

impl AppData {
    fn new(path: PathBuf) -> Result<AppData, AppDataCreationError> {
        let project_path = path
            .to_str()
            .ok_or(AppDataCreationError::Parsing)?
            .to_owned();
        let (diffs, stats) =
            git::get_diffs(project_path.clone()).map_err(|_| AppDataCreationError::Parsing)?;

        Ok(AppData {
            project_path,
            diffs,
            stats,
            selected_diff_index: 0,
        })
    }

    fn refresh(&mut self) -> Result<(), DiffParsingError> {
        let (diffs, stats) = git::get_diffs(self.project_path.clone())?;
        self.diffs = diffs;
        self.stats = stats;
        self.selected_diff_index = 0;

        Ok(())
    }

    fn get_selected_diff(&self) -> Option<&Diff> {
        self.diffs.get(self.selected_diff_index)
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.control_data.show_err_dialog {
                error_dialog(ctx, &mut self.control_data)
            }

            ui.add(SelectionAreaWidget {
                app_data: &mut self.app_data,
                control_data: &mut self.control_data,
            });

            if let Some(app_data) = &self.app_data {
                ui.add(ProjectAreaWidget::new(app_data.clone()));

                if app_data.diffs.is_empty() {
                    return;
                }
            }

            ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                if let Some(app_data) = &mut self.app_data {
                    ui.add(FilesAreaWidget { app_data });
                }

                ui.separator();
                if let Some(app_data) = &self.app_data {
                    if let Some(diff) = app_data.get_selected_diff() {
                        ui.add(DiffAreaWidget::new(diff.clone()));
                    }
                }
            });
        });
    }
}
