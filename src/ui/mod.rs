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

pub fn selection(ui: &mut Ui, ctx: &Context, control_data: &ControlData, sender: &Sender<Message>) {
    puffin::profile_function!();

    if control_data.show_err_dialog {
        error_dialog(ctx, control_data, sender);
    }

    selection_area::ui(ui, sender);
}

pub fn main(
    ui: &mut Ui,
    ctx: &Context,
    app_data: &AppData,
    control_data: &mut ControlData,
    sender: &Sender<Message>,
) {
    puffin::profile_function!();

    let diff_data = match control_data.diff_type {
        DiffType::Modified => &app_data.modified_diff_data,
        DiffType::Staged => &app_data.staged_diff_data,
    };

    ui.separator();
    ui.horizontal(|ui| {
        ui.heading(RichText::new(&app_data.project_path).color(Color32::WHITE));
        if ui.button("Log").clicked() {
            control_data.history_open = !control_data.history_open;
        }
    });
    ui.separator();

    diff_type::ui(ui, control_data);

    stats::ui(ui, &diff_data.stats);

    if diff_data.diffs.is_empty() {
        return;
    }

    ui.separator();

    ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
        files_area::ui(ui, diff_data, control_data, sender);

        ui.separator();

        if let Some(diff) = diff_data.get_diff(&control_data.selected_diff) {
            ui.vertical(|ui| {
                ui.label(control_data.selected_diff.to_str().unwrap());
                diff_area::ui(ui, &diff);
            });
        }
    });

    if control_data.history_open {
        log::ui(ctx, &app_data.commits, control_data);
    }
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
