use std::ops::Range;

use egui::{ScrollArea, Ui};

use crate::{git::diff::Diff, ui::code};

pub fn ui(ui: &mut Ui, diff: &Diff) {
    puffin::profile_function!();

    ScrollArea::both()
        .id_source("diff area")
        .auto_shrink([false, false])
        .show_rows(ui, 10.0, diff.line_count(), |ui, row_range| {
            let Range { start, end } = row_range;
            ui.horizontal(|ui| {
                code::ui(ui, diff);
            });
        });
}
