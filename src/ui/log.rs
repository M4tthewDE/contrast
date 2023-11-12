use std::sync::mpsc::Sender;

use egui::{Color32, Context, RichText, ScrollArea, Ui, Window};

use crate::{data::Message, git::Commit};

pub fn ui(ctx: &Context, sender: &Sender<Message>, commits: &Vec<Commit>) {
    let mut open = true;
    Window::new("History").open(&mut open).show(ctx, |ui| {
        ScrollArea::vertical()
            .id_source("history scroll area")
            .show(ui, |ui| {
                for commit in commits {
                    show_commit(ui, commit);
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
    ui.label(RichText::new(format!("commit {}", commit.id)).color(Color32::LIGHT_BLUE));
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
