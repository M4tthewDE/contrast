use std::ops::Range;

use egui::{Color32, Response, RichText, Ui, Widget};

use crate::git::Diff;

pub struct LineNumbersWidget {
    diff: Diff,
    range: Range<usize>,
}

impl LineNumbersWidget {
    pub fn new(diff: Diff, range: Range<usize>) -> LineNumbersWidget {
        LineNumbersWidget { diff, range }
    }
}

impl Widget for LineNumbersWidget {
    fn ui(self, ui: &mut Ui) -> Response {
        puffin::profile_function!("LineNumbersWidget");

        let lines = self.diff.lines_content.lines().collect::<Vec<&str>>();
        let Range { start, end } = self.range;
        let end = std::cmp::min(end, lines.len());

        let content = &lines[start..end].join("\n");
        ui.vertical(|ui| {
            ui.add_space(3.0);
            ui.label(RichText::new(content).monospace().color(Color32::GRAY))
        })
        .response
    }
}
