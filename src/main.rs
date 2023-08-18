use data::{AppData, ControlData};
use egui::{Align, Layout};
use ui::{error_dialog, DiffAreaWidget, FilesAreaWidget, ProjectAreaWidget, SelectionAreaWidget};

use eframe::egui;

mod data;
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
