use std::sync::mpsc::Sender;

use egui::{ScrollArea, Ui};

use crate::data::{DiffData, Message};

pub fn ui(ui: &mut Ui, diff_data: &DiffData, index: usize, sender: &Sender<Message>) {
    puffin::profile_function!("FilesAreaWidget");

    let mut index = index;
    ui.vertical(|ui| {
        ScrollArea::vertical()
            .id_source("file scroll area")
            .show(ui, |ui| {
                for (i, diff) in diff_data.diffs.iter().enumerate() {
                    if ui
                        .selectable_value(&mut index, i, diff.file_name())
                        .clicked()
                    {
                        sender
                            .send(Message::ChangeSelectedDiffIndex(i))
                            .expect("Channel closed unexpectedly!");
                    }
                }
            });
    });
}
