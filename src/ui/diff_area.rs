use std::ops::Range;

use egui::{Color32, RichText, ScrollArea, Ui};

use crate::{
    git::Diff,
    ui::{code, line_numbers, origins, LOG_AREA_WIDTH},
};

pub fn ui(ui: &mut Ui, diff: &Diff) {
    puffin::profile_function!();

    if diff.lines.is_empty() {
        ui.label(RichText::new("No content").color(Color32::GRAY));
        return;
    }

    let total_rows = diff.lines.len() + diff.headers.len();
    let scroll_width = ui.available_width() - LOG_AREA_WIDTH;

    ScrollArea::both()
        .id_source("diff area")
        .auto_shrink([false, false])
        .max_width(scroll_width)
        .show_rows(ui, 10.0, total_rows, |ui, row_range| {
            let Range { start, end } = row_range;
            ui.horizontal(|ui| {
                line_numbers::ui(ui, diff, start, end);
                origins::ui(ui, diff, start, end);
                code::ui(ui, diff, start, end);
            });
        });
}
