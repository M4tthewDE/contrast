use std::sync::mpsc::Sender;

use egui::{Color32, Context, RichText, ScrollArea, Ui, Window};

use crate::{
    data::{ControlData, Message},
    load_repository,
};

pub fn ui(ctx: &Context, ui: &mut Ui, sender: &Sender<Message>, control_data: &mut ControlData) {
    puffin::profile_function!("selection_area::ui");
    ui.horizontal(|ui| {
        ui.heading(RichText::new("Diff Viewer").color(Color32::WHITE));
        ui.separator();

        if ui
            .button(RichText::new("Open").color(Color32::WHITE))
            .clicked()
        {
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                load_repository(path, sender);
            }
        }

        ui.separator();

        if ui
            .button(RichText::new("About").color(Color32::WHITE))
            .clicked()
        {
            control_data.show_about_dialog = true;
        }

        if control_data.show_about_dialog {
            Window::new("About")
                .collapsible(false)
                .open(&mut control_data.show_about_dialog)
                .show(ctx, |ui| {
                    ui.vertical(|ui| {
                        ScrollArea::both().show(ui, |ui| {
                            ui.label(
                                RichText::new("Git Diff Viewer created by Matthias Kronberg")
                                    .color(Color32::WHITE),
                            );
                            ui.separator();
                            ui.label(RichText::new("Licenses").color(Color32::WHITE));
                            ui.label(&control_data.font_license);
                        });
                    });
                });
        }
    });
}
