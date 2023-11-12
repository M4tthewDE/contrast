use egui::{Color32, RichText, Ui};

use crate::git::Stats;

pub fn ui(ui: &mut Ui, stats: &Stats) {
    puffin::profile_function!();

    let file_changed_count = stats.files_changed;
    let insertion_count = stats.insertions;
    let deletion_count = stats.deletions;

    let files_richtext = match file_changed_count {
        1 => RichText::new(format!("{} file changed,", file_changed_count)).color(Color32::WHITE),
        _ => RichText::new(format!("{} files changed,", file_changed_count)).color(Color32::WHITE),
    };

    let insertions_richtext = match insertion_count {
        1 => RichText::new(format!("{} insertion(+),", insertion_count)).color(Color32::GREEN),
        _ => RichText::new(format!("{} insertions(+),", insertion_count)).color(Color32::GREEN),
    };

    let deletions_richtext = match deletion_count {
        1 => RichText::new(format!("{} deletion(-)", deletion_count)).color(Color32::RED),
        _ => RichText::new(format!("{} deletions(-)", deletion_count)).color(Color32::RED),
    };

    ui.horizontal(|ui| {
        ui.label(files_richtext);
        ui.label(insertions_richtext);
        ui.label(deletions_richtext);
    });
}
