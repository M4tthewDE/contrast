use egui::{Color32, Response, RichText, Ui};

use crate::git::diff::{Diff, EditType};

pub fn ui(ui: &mut Ui, diff: &Diff) -> Response {
    puffin::profile_function!("code::ui");

    ui.vertical(|ui| {
        for edit in &diff.edits {
            match edit.typ {
                EditType::Ins => ui.label(
                    RichText::new(edit.b_line.clone().unwrap().text)
                        .monospace()
                        .color(Color32::GREEN),
                ),
                EditType::Del => ui.label(
                    RichText::new(edit.a_line.clone().unwrap().text)
                        .monospace()
                        .color(Color32::RED),
                ),
                EditType::Eql => ui.label(
                    RichText::new(edit.a_line.clone().unwrap().text)
                        .monospace()
                        .color(Color32::WHITE),
                ),
            };
        }
    })
    .response
}
