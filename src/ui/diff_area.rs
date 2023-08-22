use egui::{Color32, Response, RichText, ScrollArea, Ui, Widget};

use crate::{
    git::Diff,
    ui::{code::CodeWidget, line_numbers::LineNumbersWidget, origins::OriginsWidget},
};

pub struct DiffAreaWidget {
    diff: Diff,
}

impl DiffAreaWidget {
    pub fn new(diff: Diff) -> DiffAreaWidget {
        DiffAreaWidget { diff }
    }
}

impl Widget for DiffAreaWidget {
    fn ui(self, ui: &mut Ui) -> Response {
        puffin::profile_function!("DiffAreaWidget");
        if self.diff.lines.is_empty() {
            return ui.label(RichText::new("No content").color(Color32::GRAY));
        }

        let total_rows = self.diff.lines.len() + self.diff.headers.len();

        ui.vertical(|ui| {
            ScrollArea::both()
                .id_source("diff area")
                .auto_shrink([false, false])
                .show_rows(ui, 10.0, total_rows, |ui, row_range| {
                    ui.horizontal(|ui| {
                        ui.add(LineNumbersWidget::new(self.diff.clone(), row_range.clone()));
                        ui.add(OriginsWidget::new(self.diff.clone(), row_range.clone()));
                        ui.add(CodeWidget::new(self.diff.clone(), row_range.clone()));
                    });
                });
        })
        .response
    }
}
