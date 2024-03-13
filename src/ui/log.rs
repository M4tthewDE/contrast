use std::ops::Range;

use egui::{Color32, Label, RichText, ScrollArea, Sense, Ui};

use crate::{data::ControlData, git::commit::Commit};

pub fn ui(ui: &mut Ui, commits: &[Commit], control_data: &mut ControlData) {
    puffin::profile_function!();
    ui.add_space(10.0);

    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.heading("Log");
            if ui.button("Close").clicked() {
                control_data.log_open = false;
            };
        });

        ui.horizontal(|ui| {
            ui.label("Search:");
            ui.text_edit_singleline(&mut control_data.search_string);
        });

        ui.separator();

        let commits: Vec<&Commit> = commits
            .iter()
            .filter(|c| c.contains(&control_data.search_string))
            .collect();

        // consider using https://github.com/emilk/egui/issues/1376
        ScrollArea::vertical()
            .id_source("history scroll area")
            .show_rows(ui, 100.0, commits.len(), |ui, row_range| {
                let Range { start, end } = row_range;
                for commit in &commits[start..end] {
                    show_commit(ui, commit)
                }
            });
    });
}

fn show_commit(ui: &mut Ui, commit: &Commit) {
    puffin::profile_function!();

    if ui
        .add(
            Label::new(RichText::new(commit.hash.to_string()).color(Color32::LIGHT_BLUE))
                .sense(Sense::click()),
        )
        .on_hover_text_at_pointer("Click to copy id")
        .clicked()
    {
        ui.output_mut(|po| {
            po.copied_text = commit.clone().hash;
        });
    }

    ui.label(RichText::new(format!("Author: {}", commit.author.name)).color(Color32::WHITE));
    ui.label(RichText::new(format!("Commiter: {}", commit.commiter.name)).color(Color32::WHITE));
    ui.label(RichText::new(format!("Date: {}", commit.commiter.timestamp)).color(Color32::WHITE));
    ui.add_space(10.0);
    ui.horizontal(|ui| {
        ui.add_space(10.0);
        ui.label(RichText::new(commit.message.to_string()).color(Color32::WHITE));
    });
    ui.separator();
}
