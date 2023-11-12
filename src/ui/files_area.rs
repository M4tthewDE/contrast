use egui::{Button, Color32, RichText, ScrollArea, Ui};

use crate::data::{AppData, ControlData, DiffData, DiffType, Tree};

pub fn ui(
    ui: &mut Ui,
    diff_data: &DiffData,
    control_data: &mut ControlData,
    app_data: &mut AppData,
) {
    puffin::profile_function!();

    ui.vertical(|ui| {
        ScrollArea::vertical()
            .id_source("file scroll area")
            .show(ui, |ui| {
                show_tree(ui, &diff_data.file_tree, 0, control_data, app_data);
            });
    });
}

fn show_tree(
    ui: &mut Ui,
    tree: &Tree,
    depth: usize,
    control_data: &mut ControlData,
    app_data: &mut AppData,
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
                match control_data.diff_type {
                    DiffType::Modified => {
                        app_data.modified_diff_data.file_tree.toggle_open(tree.id)
                    }
                    DiffType::Staged => app_data.modified_diff_data.file_tree.toggle_open(tree.id),
                }
            };
        });
    }

    if !tree.open {
        return;
    }

    for node in &tree.nodes {
        show_tree(ui, node, depth + 1, control_data, app_data);
    }

    for file in &tree.files {
        ui.horizontal(|ui| {
            for _ in 0..depth {
                ui.add_space(10.0);
            }

            let button = if file.path == *control_data.selected_diff {
                Button::new(
                    RichText::new(format!("🖹 {}", file.clone().get_name().unwrap()))
                        .color(Color32::WHITE),
                )
                .frame(false)
            } else {
                Button::new(format!("🖹 {}", file.clone().get_name().unwrap())).frame(false)
            };

            if ui.add(button).clicked() {
                control_data.selected_diff = file.path.clone();
            }
        });
    }
}
