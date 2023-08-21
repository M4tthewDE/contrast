use std::sync::mpsc::Sender;

use egui::Ui;

use crate::data::{DiffType, Message};

pub struct DiffTypeSelection {
    sender: Sender<Message>,
    selected_diff_type: DiffType,
}

impl DiffTypeSelection {
    pub fn new(sender: Sender<Message>, selected_diff_type: DiffType) -> DiffTypeSelection {
        DiffTypeSelection {
            sender,
            selected_diff_type,
        }
    }
}

impl DiffTypeSelection {
    pub fn ui(&mut self, ui: &mut Ui) {
        puffin::profile_function!("DiffTypeSelectionArea");
        ui.horizontal(|ui| {
            if ui
                .selectable_value(
                    &mut self.selected_diff_type,
                    DiffType::Modified,
                    DiffType::Modified.label_text(),
                )
                .clicked()
                || ui
                    .selectable_value(
                        &mut self.selected_diff_type,
                        DiffType::Staged,
                        DiffType::Staged.label_text(),
                    )
                    .clicked()
            {
                self.sender
                    .send(Message::ChangeDiffType(self.selected_diff_type.clone()))
                    .expect("Channel closed unexpectedly!");
            }
        });
    }
}
