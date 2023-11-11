use std::{path::PathBuf, sync::mpsc::Sender};

use egui::{ScrollArea, Ui};

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
    _selected_diff: &PathBuf,
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
        show_tree(ui, node, depth + 1, _selected_diff, sender);
    }

    for file in tree.files {
        ui.horizontal(|ui| {
            for _ in 0..depth {
                ui.add_space(10.0);
            }

            if ui
                .button(format!("🖹 {}", file.clone().get_name().unwrap()))
                .clicked()
            {
                sender
                    .send(Message::ChangeSelectedDiff(file.path))
                    .expect("Channel closed unexpectedly!");
            }
        });
    }
}
