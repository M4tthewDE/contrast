use std::{env, sync::mpsc::Sender};

use egui::{Align, Color32, Context, Layout, RichText, Window};

use crate::{
    data::{DiffType, Message},
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

        selection_area::ui(ui, sender);

        if let Some(app_data) = app_data {
            let diff_data = match control_data.diff_type {
                DiffType::Modified => &app_data.modified_diff_data,
                DiffType::Staged => &app_data.staged_diff_data,
            };

            ui.separator();
            ui.heading(RichText::new(&app_data.project_path).color(Color32::WHITE));
            ui.separator();

            diff_type::ui(ui, control_data.diff_type.clone(), sender);

            stats::ui(ui, &diff_data.stats);

            if diff_data.diffs.is_empty() {
                return;
            }

            ui.separator();

            ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                files_area::ui(ui, diff_data, &control_data.selected_diff, sender);

                ui.separator();

                if let Some(diff) = diff_data.get_diff(&control_data.selected_diff) {
                    diff_area::ui(ui, &diff);
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
            ui.label(RichText::new(&control_data.error_information).strong());
            if ui.button("Close").clicked() {
                sender
                    .send(Message::CloseError)
                    .expect("Channel closed unexpectedly!");
            }
        });
}
