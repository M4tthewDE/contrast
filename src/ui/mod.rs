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

const LOG_AREA_WIDTH: f32 = 300.0;

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

pub fn main(ui: &mut Ui, app_data: &mut AppData, control_data: &mut ControlData) {
    puffin::profile_function!();

    let diff_data = match control_data.diff_type {
        DiffType::Modified => app_data.modified_diff_data.clone(),
        DiffType::Staged => app_data.staged_diff_data.clone(),
    };

    ui.heading(RichText::new(&app_data.project_path).color(Color32::WHITE));
    ui.separator();

    ui.horizontal(|ui| {
        diff_type::ui(ui, control_data);
        ui.separator();
        if ui
            .button(RichText::new("Git log").color(Color32::WHITE))
            .clicked()
        {
            control_data.log_open = !control_data.log_open;
        }
    });

    ui.add_space(10.0);
    stats::ui(ui, &diff_data.stats);
    ui.separator();

    ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
        if !diff_data.diffs.is_empty() {
            files_area::ui(ui, &diff_data, control_data, app_data);

            ui.separator();

            if let Some(diff) = diff_data.get_diff(&control_data.selected_diff) {
                ui.vertical(|ui| {
                    ui.label(control_data.selected_diff.to_str().unwrap());
                    diff_area::ui(ui, &diff);
                });
            }
        }

        ui.add_space(ui.available_width() - LOG_AREA_WIDTH);

        if control_data.log_open {
            ui.separator();
            log::ui(ui, &app_data.commits, control_data);
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
