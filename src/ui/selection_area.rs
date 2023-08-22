use std::{path::PathBuf, sync::mpsc::Sender};

use egui::{Color32, RichText, Ui};

use crate::{data::Message, AppData};

pub fn ui(ui: &mut Ui, app_data: &Option<AppData>, sender: &Sender<Message>) {
    puffin::profile_function!("SelectionAreaWidget");
    ui.horizontal(|ui| {
        ui.heading(RichText::new("Diff Viewer").color(Color32::WHITE));
        ui.separator();

        if ui
            .button(RichText::new("Open").color(Color32::WHITE))
            .clicked()
        {
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                sender
                    .send(Message::LoadDiff(path))
                    .expect("Channel closed unexpectedly!");
            }
        }

        if ui
            .button(RichText::new("Refresh").color(Color32::WHITE))
            .clicked()
        {
            if let Some(app_data) = app_data {
                sender
                    .send(Message::LoadDiff(PathBuf::from(
                        app_data.project_path.clone(),
                    )))
                    .expect("Channel closed unexpectedly!");
            }
        }
    });
}