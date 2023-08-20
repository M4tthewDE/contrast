use std::{path::PathBuf, sync::mpsc::Sender};

use egui::{Color32, Response, RichText, Ui, Widget};

use crate::{data::Message, AppData};

pub struct SelectionAreaWidget {
    app_data: Option<AppData>,
    sender: Sender<Message>,
}

impl SelectionAreaWidget {
    pub fn new(app_data: Option<AppData>, sender: Sender<Message>) -> SelectionAreaWidget {
        SelectionAreaWidget { app_data, sender }
    }
}

impl Widget for SelectionAreaWidget {
    fn ui(self, ui: &mut Ui) -> Response {
        puffin::profile_function!("SelectionAreaWidget");
        ui.horizontal(|ui| {
            ui.heading(RichText::new("Diff Viewer").color(Color32::WHITE));
            ui.separator();

            if ui
                .button(RichText::new("Open").color(Color32::WHITE))
                .clicked()
            {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    self.sender
                        .send(Message::LoadDiff(path))
                        .expect("Channel closed unexpectedly!");
                }
            }

            if ui
                .button(RichText::new("Refresh").color(Color32::WHITE))
                .clicked()
            {
                if let Some(app_data) = self.app_data {
                    self.sender
                        .send(Message::LoadDiff(PathBuf::from(
                            app_data.project_path.clone(),
                        )))
                        .expect("Channel closed unexpectedly!");
                }
            }
        })
        .response
    }
}
