use std::sync::mpsc::Sender;

use egui::{ScrollArea, Ui};

use crate::data::{DiffData, Message, Tree};

pub fn ui(ui: &mut Ui, diff_data: &DiffData, index: usize, sender: &Sender<Message>) {
    puffin::profile_function!("files_area::ui");

    let file_tree = diff_data.file_tree();

    let mut index = index;
    ui.vertical(|ui| {
        ScrollArea::vertical()
            .id_source("file scroll area")
            .show(ui, |ui| {
                show_tree(ui, file_tree, 0);
                //for (i, diff) in diff_data.diffs.iter().enumerate() {
                //    if ui
                //        .selectable_value(&mut index, i, diff.file_name().to_str().unwrap())
                //        .clicked()
                //    {
                //        sender
                //            .send(Message::ChangeSelectedDiffIndex(i))
                //            .expect("Channel closed unexpectedly!");
                //    }
                //}
            });
    });
}

fn show_tree(ui: &mut Ui, tree: Tree, depth: usize) {
    if !tree.name.is_empty() {
        if tree.nodes.is_empty() {
            ui.label(tree.name);
            return;
        }

        ui.label(format!("{}/", tree.name));
    }

    for node in tree.nodes {
        show_tree(ui, node, depth + 1);
    }

    for file in tree.files {
        ui.label("PENIS");
    }
}
