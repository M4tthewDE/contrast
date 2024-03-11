use std::cmp;

use egui::{
    text::LayoutJob,
    util::cache::{ComputerMut, FrameCache},
    Color32, Context, FontFamily, FontId, Layout, Response, TextEdit, TextFormat, Ui,
};

use crate::git::diff::Diff;

pub fn ui(ui: &mut Ui, diff: &Diff, start: usize, end: usize) -> Response {
    puffin::profile_function!("code::ui");

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
        let tag = edit.typ.get_tag();
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

        let text = edit
            .a_line
            .clone()
            .unwrap_or_else(|| edit.b_line.clone().unwrap())
            .text;

        content.push_str(&format!("{tag} {old_line} {new_line}    {text}\n"));
    }

    let mut layouter = |ui: &egui::Ui, string: &str, _wrap_width: f32| {
        let layout_job: egui::text::LayoutJob = highlight(ui.ctx(), string);
        ui.fonts(|f| f.layout_job(layout_job))
    };

    ui.with_layout(Layout::left_to_right(egui::Align::Min), |ui| {
        ui.add(
            TextEdit::multiline(&mut content.as_str())
                .desired_width(f32::INFINITY)
                .frame(false)
                .code_editor()
                .layouter(&mut layouter),
        );
    })
    .response
}

type HighlightCache = FrameCache<LayoutJob, LayoutHandler>;

fn highlight(ctx: &Context, text: &str) -> LayoutJob {
    impl ComputerMut<&str, LayoutJob> for LayoutHandler {
        fn compute(&mut self, text: &str) -> LayoutJob {
            puffin::profile_function!();
            LayoutHandler::layout_job(text)
        }
    }

    ctx.memory_mut(|mem| mem.caches.cache::<HighlightCache>().get(text))
}

#[derive(Debug, Default)]
struct LayoutHandler;

impl LayoutHandler {
    fn layout_job(text: &str) -> LayoutJob {
        puffin::profile_function!();

        let mut job = LayoutJob::default();
        job.wrap.max_width = f32::INFINITY;

        for line in text.lines() {
            let format = if let Some(tag) = line.get(0..1) {
                match tag {
                    "+" => {
                        TextFormat::simple(FontId::new(12.0, FontFamily::Monospace), Color32::GREEN)
                    }
                    "-" => {
                        TextFormat::simple(FontId::new(12.0, FontFamily::Monospace), Color32::RED)
                    }
                    _ => {
                        TextFormat::simple(FontId::new(12.0, FontFamily::Monospace), Color32::WHITE)
                    }
                }
            } else {
                TextFormat::simple(FontId::new(12.0, FontFamily::Monospace), Color32::WHITE)
            };

            job.append(format!("{line}\n").as_str(), 0.0, format.clone());
        }

        job
    }
}
