use std::sync::mpsc::Sender;

use egui::{Align, Color32, Context, Layout, RichText, Ui, Window};

use crate::{
    data::{DiffType, Message},
    AppData, ControlData,
};

mod code;
mod diff_area;
mod diff_type;
mod files_area;
mod line_numbers;
mod log;
mod origins;
mod selection_area;
mod stats;

pub fn selection(
    ui: &mut Ui,
    ctx: &Context,
    control_data: &mut ControlData,
    sender: &Sender<Message>,
) {
    puffin::profile_function!();

    if control_data.show_err_dialog {
        error_dialog(ctx, control_data);
    }

    selection_area::ui(ui, sender);
}

pub fn main(ui: &mut Ui, ctx: &Context, app_data: &mut AppData, control_data: &mut ControlData) {
    puffin::profile_function!();

    let diff_data = match control_data.diff_type {
        DiffType::Modified => app_data.modified_diff_data.clone(),
        DiffType::Staged => app_data.staged_diff_data.clone(),
    };

    ui.separator();
    ui.horizontal(|ui| {
        ui.heading(RichText::new(&app_data.project_path).color(Color32::WHITE));
        if ui.button("Log").clicked() {
            control_data.history_open = true;
        }
    });
    ui.separator();

    diff_type::ui(ui, control_data);

    stats::ui(ui, &diff_data.stats);

    if control_data.history_open {
        log::ui(ctx, &app_data.commits, control_data);
    }

    if diff_data.diffs.is_empty() {
        return;
    }

    ui.separator();

    ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
        files_area::ui(ui, &diff_data, control_data, app_data);

        ui.separator();

        if let Some(diff) = diff_data.get_diff(&control_data.selected_diff) {
            ui.vertical(|ui| {
                ui.label(control_data.selected_diff.to_str().unwrap());
                diff_area::ui(ui, &diff);
            });
        }
    });
}

pub fn error_dialog(ctx: &Context, control_data: &mut ControlData) {
    Window::new("Error")
        .collapsible(false)
        .resizable(true)
        .show(ctx, |ui| {
            ui.label(RichText::new(&control_data.error_information).strong());
            if ui.button("Close").clicked() {
                control_data.error_information = "".to_string();
                control_data.show_err_dialog = false;
            }
        });
}
