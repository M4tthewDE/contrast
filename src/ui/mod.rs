use std::{env, path::PathBuf, sync::mpsc::Sender};

use egui::{Align, Color32, Context, Layout, Response, RichText, ScrollArea, Ui, Widget, Window};

use crate::{
    data::{DiffData, DiffType, Message},
    ui::{diff_area::DiffAreaWidget, diff_type::DiffTypeSelection, stats::StatsWidget},
    AppData, ControlData,
};

mod code;
mod diff_area;
mod diff_type;
mod line_numbers;
mod origins;
mod stats;

pub fn show(
    ctx: &Context,
    app_data: &Option<AppData>,
    control_data: &ControlData,
    sender: &Sender<Message>,
) {
    egui::CentralPanel::default().show(ctx, |ui| {
        puffin::profile_function!();

        if control_data.show_err_dialog {
            error_dialog(ctx, control_data, sender);
        }

        if env::var("PROFILING").is_ok() {
            puffin_egui::profiler_window(ctx);
        }

        ui.add(SelectionAreaWidget { app_data, sender });

        if let Some(app_data) = app_data {
            let diff_data = match control_data.diff_type {
                DiffType::Modified => &app_data.modified_diff_data,
                DiffType::Staged => &app_data.staged_diff_data,
            };

            ui.separator();
            ui.heading(RichText::new(app_data.project_path.clone()).color(Color32::WHITE));
            ui.separator();

            ui.add(DiffTypeSelection::new(
                sender.clone(),
                &mut control_data.diff_type.clone(),
            ));

            ui.add(StatsWidget::new(diff_data.stats.clone()));

            if diff_data.diffs.is_empty() {
                return;
            }

            ui.separator();

            ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                ui.add(FilesAreaWidget {
                    diff_data,
                    selected_diff_index: control_data.selected_diff_index,
                    sender,
                });
                ui.separator();

                if let Some(diff) = diff_data.diffs.get(control_data.selected_diff_index) {
                    ui.add(DiffAreaWidget::new(diff.clone()));
                }
            });
        }
    });
}

pub struct SelectionAreaWidget<'a> {
    app_data: &'a Option<AppData>,
    sender: &'a Sender<Message>,
}

impl Widget for SelectionAreaWidget<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        puffin::profile_function!("SelectionAreaWidget");
        ui.horizontal(|ui| {
            ui.heading(RichText::new("Diff Viewer").color(Color32::WHITE));
            ui.separator();

            if ui
                .button(RichText::new("Open").color(Color32::WHITE))
                .clicked()
            {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    self.sender
                        .send(Message::LoadDiff(path))
                        .expect("Channel closed unexpectedly!");
                }
            }

            if ui
                .button(RichText::new("Refresh").color(Color32::WHITE))
                .clicked()
            {
                if let Some(app_data) = self.app_data {
                    self.sender
                        .send(Message::LoadDiff(PathBuf::from(
                            app_data.project_path.clone(),
                        )))
                        .expect("Channel closed unexpectedly!");
                }
            }
        })
        .response
    }
}

pub fn error_dialog(ctx: &Context, control_data: &ControlData, sender: &Sender<Message>) {
    Window::new("Error")
        .collapsible(false)
        .resizable(true)
        .show(ctx, |ui| {
            ui.label(RichText::new(control_data.error_information.clone()).strong());
            if ui.button("Close").clicked() {
                sender
                    .send(Message::CloseError)
                    .expect("Channel closed unexpectedly!");
            }
        });
}

pub struct FilesAreaWidget<'a> {
    diff_data: &'a DiffData,
    selected_diff_index: usize,
    sender: &'a Sender<Message>,
}

impl Widget for FilesAreaWidget<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        puffin::profile_function!("FilesAreaWidget");
        ui.vertical(|ui| {
            ScrollArea::vertical()
                .id_source("file scroll area")
                .show(ui, |ui| {
                    for (i, diff) in self.diff_data.diffs.iter().enumerate() {
                        if self.selected_diff_index == i {
                            ui.button(diff.file_name()).highlight();
                        } else if ui.button(diff.file_name()).clicked() {
                            self.sender
                                .send(Message::ChangeSelectedDiffIndex(i))
                                .expect("Channel closed unexpectedly!");
                        }
                    }
                });
        })
        .response
    }
}
