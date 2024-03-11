use egui::{ScrollArea, Ui};

use crate::{git::diff::Diff, ui::code};

pub fn ui(ui: &mut Ui, diff: &Diff) {
    puffin::profile_function!();

    ScrollArea::both()
        .id_source("diff area")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                code::ui(ui, diff);
            });
        });
}
