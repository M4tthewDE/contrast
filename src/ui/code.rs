use std::cmp;

use egui::{
    text::LayoutJob,
    util::cache::{ComputerMut, FrameCache},
    Color32, Context, FontFamily, FontId, Layout, Response, TextEdit, TextFormat, Ui,
};

use crate::git::diff::FileDiff;

pub fn ui(ui: &mut Ui, diff: &FileDiff, start: usize, end: usize) -> Response {
    puffin::profile_function!("code::ui");

    let mut content = String::new();
    for edit in &diff.edits[start..cmp::min(end, diff.edits.len())] {
        let tag = edit.typ.get_tag();

        let text = edit
            .a_line
            .clone()
            .unwrap_or_else(|| edit.b_line.clone().unwrap())
            .text;

        content.push_str(&format!("{tag}{text}\n"));
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

            job.append(format!("{}\n", &line[1..]).as_str(), 0.0, format.clone());
        }

        job
    }
}
