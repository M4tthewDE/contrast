use std::sync::mpsc::Sender;

use egui::{ScrollArea, Ui};

use crate::data::{DiffData, Message};

pub struct FilesArea {
    diff_data: DiffData,
    selected_diff_index: usize,
    sender: Sender<Message>,
}

impl FilesArea {
    pub fn new(
        diff_data: DiffData,
        selected_diff_index: usize,
        sender: Sender<Message>,
    ) -> FilesArea {
        FilesArea {
            diff_data,
            selected_diff_index,
            sender,
        }
    }
}

impl FilesArea {
    pub fn ui(&mut self, ui: &mut Ui) {
        puffin::profile_function!("FilesAreaWidget");
        ui.vertical(|ui| {
            ScrollArea::vertical()
                .id_source("file scroll area")
                .show(ui, |ui| {
                    for (i, diff) in self.diff_data.diffs.iter().enumerate() {
                        if ui
                            .selectable_value(&mut self.selected_diff_index, i, diff.file_name())
                            .clicked()
                        {
                            self.sender
                                .send(Message::ChangeSelectedDiffIndex(i))
                                .expect("Channel closed unexpectedly!");
                        }
                    }
                });
        });
    }
}
