use std::sync::mpsc::Sender;

use egui::{Color32, RichText, Ui};

use crate::data::Message;

pub fn ui(ui: &mut Ui, sender: &Sender<Message>) {
    puffin::profile_function!("selection_area::ui");
    ui.horizontal(|ui| {
        ui.heading(RichText::new("Diff Viewer").color(Color32::WHITE));
        ui.separator();

        if ui
            .button(RichText::new("Open").color(Color32::WHITE))
            .clicked()
        {
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                sender
                    .send(Message::LoadRepository(path))
                    .expect("Channel closed unexpectedly!");
            }
        }
    });
}
