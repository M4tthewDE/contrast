use std::sync::mpsc::Sender;

use egui::{Color32, Context, Label, RichText, ScrollArea, Sense, Ui, Window};

use crate::{data::Message, git::Commit};

pub fn ui(
    ctx: &Context,
    sender: &Sender<Message>,
    commits: &Vec<Commit>,
    search_string: &mut String,
) {
    let mut open = true;
    Window::new("History").open(&mut open).show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label("Search:");
            ui.text_edit_singleline(search_string);
        });

        ScrollArea::vertical()
            .id_source("history scroll area")
            .show(ui, |ui| {
                for commit in commits {
                    if commit.contains(search_string) {
                        show_commit(ui, commit);
                    }
                }
            });
    });

    if !open {
        sender
            .send(Message::ToggleHistory)
            .expect("Channel closed unexpectedly!");
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
