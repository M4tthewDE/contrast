use std::sync::mpsc::Sender;

use egui::Ui;

use crate::data::{DiffType, Message};

pub fn ui(ui: &mut Ui, diff_type: DiffType, sender: &Sender<Message>) {
    puffin::profile_function!("DiffTypeSelectionArea");

    let mut selected_diff_type = diff_type;
    ui.horizontal(|ui| {
        if ui
            .selectable_value(
                &mut selected_diff_type,
                DiffType::Modified,
                DiffType::Modified.label_text(),
            )
            .clicked()
            || ui
                .selectable_value(
                    &mut selected_diff_type,
                    DiffType::Staged,
                    DiffType::Staged.label_text(),
                )
                .clicked()
        {
            sender
                .send(Message::ChangeDiffType(selected_diff_type.clone()))
                .expect("Channel closed unexpectedly!");
        }
    });
}
