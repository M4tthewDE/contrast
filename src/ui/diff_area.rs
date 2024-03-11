use std::cmp;
use std::ops::Range;

use egui::{
    text::LayoutJob,
    util::cache::{ComputerMut, FrameCache},
    Color32, Context, FontFamily, FontId, Response, RichText, ScrollArea, TextEdit, TextFormat, Ui,
};

use crate::{git::diff::Diff, ui::code};

pub fn ui(ui: &mut Ui, diff: &Diff) {
    puffin::profile_function!();

    ScrollArea::both()
        .id_source("diff area")
        .auto_shrink([false, false])
        .show_rows(ui, 10.0, diff.edits.len(), |ui, row_range| {
            let Range { start, end } = row_range;
            ui.horizontal(|ui| {
                line_numbers(ui, diff, start, end);
                tag(ui, diff, start, end);
                code::ui(ui, diff, start, end);
            });
        });
}

pub fn line_numbers(ui: &mut Ui, diff: &Diff, start: usize, end: usize) -> Response {
    puffin::profile_function!("line_numbers");

    let mut max_old_line = 1;
    let mut max_new_line = 1;

    for edit in &diff.edits {
        max_old_line = cmp::max(
            max_old_line,
            edit.a_line.clone().map(|a| a.number).unwrap_or_default(),
        );
        max_new_line = cmp::max(
            max_new_line,
            edit.b_line.clone().map(|b| b.number).unwrap_or_default(),
        );
    }

    let max_old_line = max_old_line.to_string().len();
    let max_new_line = max_new_line.to_string().len();

    let mut content = String::new();
    for edit in &diff.edits[start..cmp::min(end, diff.edits.len())] {
        let mut old_line = edit
            .a_line
            .clone()
            .map(|a| a.number.to_string())
            .unwrap_or(" ".to_string());
        let mut new_line = edit
            .b_line
            .clone()
            .map(|b| b.number.to_string())
            .unwrap_or(" ".to_string());

        while old_line.len() < max_old_line {
            old_line = " ".to_owned() + &old_line;
        }

        while new_line.len() < max_new_line {
            new_line = " ".to_owned() + &new_line;
        }

        content.push_str(&format!("{old_line} {new_line}\n"));
    }

    ui.vertical(|ui| {
        ui.add_space(3.0);
        ui.label(RichText::new(content).monospace().color(Color32::GRAY))
    })
    .response
}

pub fn tag(ui: &mut Ui, diff: &Diff, start: usize, end: usize) {
    puffin::profile_function!("tag");

    let mut layouter = |ui: &egui::Ui, string: &str, _wrap_width: f32| {
        let layout_job: egui::text::LayoutJob = tag_highlight(ui.ctx(), string);
        ui.fonts(|f| f.layout_job(layout_job))
    };

    let mut content = String::new();
    for edit in &diff.edits[start..cmp::min(end, diff.edits.len())] {
        content.push_str(&format!("{}\n", edit.typ.get_tag()));
    }
    ui.add(
        TextEdit::multiline(&mut content)
            .desired_width(0.0)
            .frame(false)
            .interactive(false)
            .layouter(&mut layouter),
    );
}

type TagHighlightCache = FrameCache<LayoutJob, TagLayoutHandler>;

fn tag_highlight(ctx: &Context, text: &str) -> LayoutJob {
    impl ComputerMut<&str, LayoutJob> for TagLayoutHandler {
        fn compute(&mut self, text: &str) -> LayoutJob {
            puffin::profile_function!();
            TagLayoutHandler::layout_job(text)
        }
    }

    ctx.memory_mut(|mem| mem.caches.cache::<TagHighlightCache>().get(text))
}
#[derive(Debug, Default)]
struct TagLayoutHandler {}

impl TagLayoutHandler {
    fn layout_job(text: &str) -> LayoutJob {
        puffin::profile_function!();

        let mut job = LayoutJob::default();
        job.wrap.max_width = f32::INFINITY;

        let insertion_format =
            TextFormat::simple(FontId::new(12.0, FontFamily::Monospace), Color32::GREEN);
        let deletion_format =
            TextFormat::simple(FontId::new(12.0, FontFamily::Monospace), Color32::RED);
        let neutral_format =
            TextFormat::simple(FontId::new(12.0, FontFamily::Monospace), Color32::WHITE);

        for line in text.split('\n') {
            if line.contains('+') {
                job.append(format!("{line}\n").as_str(), 0.0, insertion_format.clone());
            }
            if line.contains('-') {
                job.append(format!("{line}\n").as_str(), 0.0, deletion_format.clone());
            }
            if !line.contains('+') && !line.contains('-') {
                job.append(format!("{line}\n").as_str(), 0.0, neutral_format.clone());
            }
        }

        job
    }
}
