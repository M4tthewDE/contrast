use std::{path::PathBuf, sync::mpsc::Sender};

use egui::{Button, Color32, RichText, ScrollArea, Ui};

use crate::data::{DiffData, Message, Tree};

pub fn ui(ui: &mut Ui, diff_data: &DiffData, selected_diff: &PathBuf, sender: &Sender<Message>) {
    puffin::profile_function!("files_area::ui");

    ui.vertical(|ui| {
        ScrollArea::vertical()
            .id_source("file scroll area")
            .show(ui, |ui| {
                show_tree(ui, &diff_data.file_tree, 0, selected_diff, sender);
            });
    });
}

fn show_tree(
    ui: &mut Ui,
    tree: &Tree,
    depth: usize,
    selected_diff: &PathBuf,
    sender: &Sender<Message>,
) {
    if !tree.name.is_empty() {
        ui.horizontal(|ui| {
            let button = if tree.open {
                Button::new(format!("🗁 {}", tree.name)).frame(false)
            } else {
                Button::new(format!("🗀 {}", tree.name)).frame(false)
            };

            for _ in 0..depth - 1 {
                ui.add_space(10.0);
            }
            if ui.add(button).clicked() {
                sender
                    .send(Message::ToggleFolder(tree.id))
                    .expect("Channel closed unexpectedly!");
            };
        });
    }

    if !tree.open {
        return;
    }

    for node in &tree.nodes {
        show_tree(ui, node, depth + 1, selected_diff, sender);
    }

    for file in &tree.files {
        ui.horizontal(|ui| {
            for _ in 0..depth {
                ui.add_space(10.0);
            }

            let button = if file.path == *selected_diff {
                Button::new(
                    RichText::new(format!("🖹 {}", file.clone().get_name().unwrap()))
                        .color(Color32::WHITE),
                )
                .frame(false)
            } else {
                Button::new(format!("🖹 {}", file.clone().get_name().unwrap())).frame(false)
            };

            if ui.add(button).clicked() {
                sender
                    .send(Message::ChangeSelectedDiff(file.path.to_owned()))
                    .expect("Channel closed unexpectedly!");
            }
        });
    }
}
