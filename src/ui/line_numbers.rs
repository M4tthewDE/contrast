use egui::{Color32, Response, RichText, Ui};

use crate::git::Diff;

pub fn ui(ui: &mut Ui, diff: &Diff, start: usize, end: usize) -> Response {
    puffin::profile_function!("line_numbers::ui");

    let lines = diff.line_numbers.lines().collect::<Vec<&str>>();
    let end = std::cmp::min(end, lines.len());

    let content = &lines[start..end].join("\n");
    ui.vertical(|ui| {
        ui.add_space(3.0);
        ui.label(RichText::new(content).monospace().color(Color32::GRAY))
    })
    .response
}
