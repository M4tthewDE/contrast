use egui::Ui;

use crate::data::{ControlData, DiffType};

pub fn ui(ui: &mut Ui, control_data: &mut ControlData) {
    puffin::profile_function!("diff_type::ui");

    let mut selected_diff_type = control_data.diff_type.clone();
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
            control_data.diff_type = selected_diff_type;
        }
    });
}
