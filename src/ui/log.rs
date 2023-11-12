use egui::{Color32, Context, Label, RichText, ScrollArea, Sense, Ui, Window};

use crate::{data::ControlData, git::Commit};

pub fn ui(ctx: &Context, commits: &Vec<Commit>, control_data: &mut ControlData) {
    let mut open = true;
    Window::new("History").open(&mut open).show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label("Search:");
            ui.text_edit_singleline(&mut control_data.search_string);
        });

        ScrollArea::vertical()
            .id_source("history scroll area")
            .show(ui, |ui| {
                for commit in commits {
                    if commit.contains(&control_data.search_string) {
                        show_commit(ui, commit);
                    }
                }
            });
    });

    if !open {
        control_data.history_open = !control_data.history_open;
    }
}

fn show_commit(ui: &mut Ui, commit: &Commit) {
    if ui
        .add(
            Label::new(RichText::new(format!("commit {}", commit.id)).color(Color32::LIGHT_BLUE))
                .sense(Sense::click()),
        )
        .on_hover_text_at_pointer("Click to copy id")
        .clicked()
    {
        ui.output_mut(|po| {
            po.copied_text = commit.clone().id;
        });
    }

    ui.label(
        RichText::new(format!(
            "Author: {} <{}>",
            commit.author.name, commit.author.email
        ))
        .color(Color32::WHITE),
    );
    ui.label(RichText::new(format!("Date: {}", commit.time)).color(Color32::WHITE));
    ui.add_space(10.0);
    ui.horizontal(|ui| {
        ui.add_space(10.0);
        ui.label(RichText::new(commit.message.to_string()).color(Color32::WHITE));
    });
    ui.separator();
}
