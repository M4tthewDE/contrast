use egui::{
    text::LayoutJob,
    util::cache::{ComputerMut, FrameCache},
    Color32, Context, FontFamily, FontId, TextEdit, TextFormat, Ui,
};

use crate::git::Diff;

pub fn ui(ui: &mut Ui, diff: &Diff, start: usize, end: usize) {
    puffin::profile_function!("origins::ui");

    let mut layouter = |ui: &egui::Ui, string: &str, _wrap_width: f32| {
        let layout_job: egui::text::LayoutJob = origins_highlight(ui.ctx(), string);
        ui.fonts(|f| f.layout_job(layout_job))
    };

    let lines = diff.origins_content.lines().collect::<Vec<&str>>();
    let end = std::cmp::min(end, lines.len());

    let mut content = lines[start..end].join("\n");
    ui.add(
        TextEdit::multiline(&mut content)
            .desired_width(0.0)
            .frame(false)
            .interactive(false)
            .layouter(&mut layouter),
    );
}

type OriginsHighlightCache = FrameCache<LayoutJob, OriginsLayoutHandler>;

fn origins_highlight(ctx: &Context, text: &str) -> LayoutJob {
    impl ComputerMut<&str, LayoutJob> for OriginsLayoutHandler {
        fn compute(&mut self, text: &str) -> LayoutJob {
            puffin::profile_function!();
            OriginsLayoutHandler::layout_job(text)
        }
    }

    ctx.memory_mut(|mem| mem.caches.cache::<OriginsHighlightCache>().get(text))
}
#[derive(Debug, Default)]
struct OriginsLayoutHandler {}

impl OriginsLayoutHandler {
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
