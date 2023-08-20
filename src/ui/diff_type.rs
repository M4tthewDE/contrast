use std::sync::mpsc::Sender;

use egui::{ComboBox, Response, Ui, Widget};

use crate::data::{DiffType, Message};

pub struct DiffTypeSelection<'a> {
    sender: Sender<Message>,
    selected_diff_type: &'a mut DiffType,
}

impl<'a> DiffTypeSelection<'a> {
    pub fn new(
        sender: Sender<Message>,
        selected_diff_type: &'a mut DiffType,
    ) -> DiffTypeSelection<'a> {
        DiffTypeSelection {
            sender,
            selected_diff_type,
        }
    }
}

impl Widget for DiffTypeSelection<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        puffin::profile_function!("DiffTypeSelectionArea");
        ui.horizontal(|ui| {
            ui.label("Type");
            ComboBox::from_id_source("Diff Type")
                .selected_text(self.selected_diff_type.label_text())
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_value(
                            self.selected_diff_type,
                            DiffType::Modified,
                            DiffType::Modified.label_text(),
                        )
                        .clicked()
                    {
                        self.sender
                            .send(Message::ChangeDiffType(DiffType::Modified))
                            .expect("Channel closed unexpectedly!");
                    };
                    if ui
                        .selectable_value(
                            self.selected_diff_type,
                            DiffType::Staged,
                            DiffType::Staged.label_text(),
                        )
                        .clicked()
                    {
                        self.sender
                            .send(Message::ChangeDiffType(DiffType::Staged))
                            .expect("Channel closed unexpectedly!");
                    };
                });
        })
        .response
    }
}
