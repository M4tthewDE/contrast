use std::cmp;
use std::ops::Range;

use egui::{Color32, Response, RichText, ScrollArea, Ui};

use crate::{git::diff::Diff, ui::code};

pub fn ui(ui: &mut Ui, diff: &Diff) {
    puffin::profile_function!();

    ScrollArea::both()
        .id_source("diff area")
        .auto_shrink([false, false])
        .show_rows(ui, 10.0, diff.edits.len(), |ui, row_range| {
            let Range { start, end } = row_range;
            ui.horizontal(|ui| {
                line_numbers(ui, diff, start, end);
                code::ui(ui, diff, start, end);
            });
        });
}

pub fn line_numbers(ui: &mut Ui, diff: &Diff, start: usize, end: usize) -> Response {
    puffin::profile_function!("line_numbers");

    let mut max_old_line = 1;
    let mut max_new_line = 1;

    for edit in &diff.edits {
        max_old_line = cmp::max(
            max_old_line,
            edit.a_line.clone().map(|a| a.number).unwrap_or_default(),
        );
        max_new_line = cmp::max(
            max_new_line,
            edit.b_line.clone().map(|b| b.number).unwrap_or_default(),
        );
    }

    let max_old_line = max_old_line.to_string().len();
    let max_new_line = max_new_line.to_string().len();

    let mut content = String::new();
    for edit in &diff.edits[start..cmp::min(end, diff.edits.len())] {
        let mut old_line = edit
            .a_line
            .clone()
            .map(|a| a.number.to_string())
            .unwrap_or(" ".to_string());
        let mut new_line = edit
            .b_line
            .clone()
            .map(|b| b.number.to_string())
            .unwrap_or(" ".to_string());

        while old_line.len() < max_old_line {
            old_line = " ".to_owned() + &old_line;
        }

        while new_line.len() < max_new_line {
            new_line = " ".to_owned() + &new_line;
        }

        content.push_str(&format!("{old_line} {new_line}\n"));
    }

    ui.vertical(|ui| {
        ui.add_space(3.0);
        ui.label(RichText::new(content).monospace().color(Color32::GRAY))
    })
    .response
}
