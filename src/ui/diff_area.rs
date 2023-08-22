use std::ops::Range;

use egui::{Color32, RichText, ScrollArea, Ui};

use crate::{
    git::Diff,
    ui::{code, line_numbers, origins},
};

pub fn ui(ui: &mut Ui, diff: Diff) {
    puffin::profile_function!("diff_area::ui");

    if diff.lines.is_empty() {
        ui.label(RichText::new("No content").color(Color32::GRAY));
        return;
    }

    let total_rows = diff.lines.len() + diff.headers.len();

    ui.vertical(|ui| {
        ScrollArea::both()
            .id_source("diff area")
            .auto_shrink([false, false])
            .show_rows(ui, 10.0, total_rows, |ui, row_range| {
                let Range { start, end } = row_range;
                ui.horizontal(|ui| {
                    line_numbers::ui(ui, &diff, start, end);
                    origins::ui(ui, &diff, start, end);
                    code::ui(ui, &diff, start, end);
                });
            });
    });
}
