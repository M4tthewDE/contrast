use egui::{Align, Color32, Layout, RichText, ScrollArea, Ui, Window};
use std::path::PathBuf;
use ui::{DiffAreaWidget, ProjectAreaWidget};

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

    fn get_stats_richtext(&self) -> RichText {
        let file_changed_count = self.stats.files_changed;
        let insertion_count = self.stats.insertions;
        let deletion_count = self.stats.deletions;

        let content = format!(
            "{} file(s) changed, {} insertions(+), {} deletions(-)\n",
            file_changed_count, insertion_count, deletion_count
        );

        RichText::new(content).color(Color32::WHITE)
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.selection_area(ctx, ui);

            if let Some(app_data) = &self.app_data {
                ui.add(ProjectAreaWidget::new(app_data.clone()));

                if app_data.diffs.is_empty() {
                    return;
                }
            }

            ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                self.files_area(ui);

                if let Some(app_data) = &self.app_data {
                    if let Some(diff) = app_data.get_selected_diff() {
                        ui.add(DiffAreaWidget::new(diff.clone()));
                    }
                }
            });
        });
    }
}

impl MyApp {
    fn selection_area(&mut self, ctx: &egui::Context, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading(RichText::new("Diff Viewer").color(Color32::WHITE));
            ui.separator();

            if ui
                .button(RichText::new("Open").color(Color32::WHITE))
                .clicked()
            {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    match AppData::new(path) {
                        Ok(app_data) => self.app_data = Some(app_data),
                        Err(err) => match err {
                            AppDataCreationError::Parsing => {
                                self.show_error("Parsing failed!".to_owned())
                            }
                        },
                    }
                }
            }

            if self.show_err_dialog {
                self.error_dialog(ctx);
            }

            if self.app_data.is_some()
                && ui
                    .button(RichText::new("Refresh").color(Color32::WHITE))
                    .clicked()
            {
                if let Some(app_data) = &mut self.app_data {
                    if app_data.refresh().is_err() {
                        self.show_error("Refresh failed!".to_owned());
                    };
                }
            }
        });

        ui.separator();
    }

    fn files_area(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            if let Some(app_data) = &mut self.app_data {
                ScrollArea::vertical()
                    .id_source("file scroll area")
                    .show(ui, |ui| {
                        for (i, diff) in app_data.diffs.iter().enumerate() {
                            if app_data.selected_diff_index == i {
                                ui.button(diff.file_name()).highlight();
                            } else if ui.button(diff.file_name()).clicked() {
                                app_data.selected_diff_index = i;
                            }
                        }
                    });
            }
        });
    }

    fn show_error(&mut self, information: String) {
        self.error_information = information;
        self.show_err_dialog = true;
    }

    fn error_dialog(&mut self, ctx: &egui::Context) {
        Window::new("Error")
            .collapsible(false)
            .resizable(true)
            .show(ctx, |ui| {
                ui.label(RichText::new(self.error_information.clone()).strong());
                if ui.button("Close").clicked() {
                    self.error_information = "".to_owned();
                    self.show_err_dialog = false;
                }
            });
    }
}
