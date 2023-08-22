use egui::{Color32, RichText, ScrollArea, Ui};

use crate::{
    git::Diff,
    ui::{code::CodeWidget, line_numbers::LineNumbersWidget, origins::OriginsWidget},
};

pub fn ui(ui: &mut Ui, diff: Diff) {
    puffin::profile_function!("DiffAreaWidget");
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
                ui.horizontal(|ui| {
                    ui.add(LineNumbersWidget::new(diff.clone(), row_range.clone()));
                    ui.add(OriginsWidget::new(diff.clone(), row_range.clone()));
                    ui.add(CodeWidget::new(diff.clone(), row_range.clone()));
                });
            });
    });
}
