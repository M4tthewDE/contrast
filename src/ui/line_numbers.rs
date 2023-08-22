use std::ops::Range;

use egui::{Color32, Response, RichText, Ui};

use crate::git::Diff;

pub fn ui(ui: &mut Ui, diff: Diff, range: Range<usize>) -> Response {
    puffin::profile_function!("LineNumbersWidget");

    let lines = diff.lines_content.lines().collect::<Vec<&str>>();
    let Range { start, end } = range;
    let end = std::cmp::min(end, lines.len());

    let content = &lines[start..end].join("\n");
    ui.vertical(|ui| {
        ui.add_space(3.0);
        ui.label(RichText::new(content).monospace().color(Color32::GRAY))
    })
    .response
}
