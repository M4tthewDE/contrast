use std::{env, sync::mpsc::Sender};

use egui::{Align, Color32, Context, Layout, RichText, Window};

use crate::{
    data::{DiffType, Message},
    ui::{
        diff_area::DiffAreaWidget, diff_type::DiffTypeSelection, files_area::FilesAreaWidget,
        selection_area::SelectionAreaWidget, stats::StatsWidget,
    },
    AppData, ControlData,
};

mod code;
mod diff_area;
mod diff_type;
mod files_area;
mod line_numbers;
mod origins;
mod selection_area;
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

        ui.add(SelectionAreaWidget::new(app_data.clone(), sender.clone()));

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
                ui.add(FilesAreaWidget::new(
                    diff_data.clone(),
                    control_data.selected_diff_index,
                    sender.clone(),
                ));
                ui.separator();

                if let Some(diff) = diff_data.diffs.get(control_data.selected_diff_index) {
                    ui.add(DiffAreaWidget::new(diff.clone()));
                }
            });
        }
    });
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
