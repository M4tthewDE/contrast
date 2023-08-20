use std::sync::mpsc::Sender;

use egui::{Response, ScrollArea, Ui, Widget};

use crate::data::{DiffData, Message};

pub struct FilesAreaWidget {
    diff_data: DiffData,
    selected_diff_index: usize,
    sender: Sender<Message>,
}

impl FilesAreaWidget {
    pub fn new(
        diff_data: DiffData,
        selected_diff_index: usize,
        sender: Sender<Message>,
    ) -> FilesAreaWidget {
        FilesAreaWidget {
            diff_data,
            selected_diff_index,
            sender,
        }
    }
}

impl Widget for FilesAreaWidget {
    fn ui(self, ui: &mut Ui) -> Response {
        puffin::profile_function!("FilesAreaWidget");
        ui.vertical(|ui| {
            ScrollArea::vertical()
                .id_source("file scroll area")
                .show(ui, |ui| {
                    for (i, diff) in self.diff_data.diffs.iter().enumerate() {
                        if self.selected_diff_index == i {
                            ui.button(diff.file_name()).highlight();
                        } else if ui.button(diff.file_name()).clicked() {
                            self.sender
                                .send(Message::ChangeSelectedDiffIndex(i))
                                .expect("Channel closed unexpectedly!");
                        }
                    }
                });
        })
        .response
    }
}
