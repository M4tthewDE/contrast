use std::{path::PathBuf, sync::mpsc::Sender};

use egui::{Button, ScrollArea, Ui};

use crate::data::{DiffData, Message, Tree};

pub fn ui(ui: &mut Ui, diff_data: &DiffData, selected_diff: &PathBuf, sender: &Sender<Message>) {
    puffin::profile_function!("files_area::ui");

    let file_tree = diff_data.file_tree();

    ui.vertical(|ui| {
        ScrollArea::vertical()
            .id_source("file scroll area")
            .show(ui, |ui| {
                show_tree(ui, file_tree, 0, selected_diff, sender);
            });
    });
}

fn show_tree(
    ui: &mut Ui,
    tree: Tree,
    depth: usize,
    selected_diff: &PathBuf,
    sender: &Sender<Message>,
) {
    if !tree.name.is_empty() {
        ui.horizontal(|ui| {
            for _ in 0..depth - 1 {
                ui.add_space(10.0);
            }
            ui.label(format!("🗀 {}", tree.name));
        });
    }

    for node in tree.nodes {
        show_tree(ui, node, depth + 1, selected_diff, sender);
    }

    for file in tree.files {
        ui.horizontal(|ui| {
            for _ in 0..depth {
                ui.add_space(10.0);
            }

            let button = if file.path == *selected_diff {
                Button::new(format!("🖹 {}", file.clone().get_name().unwrap())).selected(true)
            } else {
                Button::new(format!("🖹 {}", file.clone().get_name().unwrap()))
            };

            if ui.add(button).clicked() {
                sender
                    .send(Message::ChangeSelectedDiff(file.path))
                    .expect("Channel closed unexpectedly!");
            }
        });
    }
}
