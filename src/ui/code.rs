use std::ops::Range;

use egui::{
    text::LayoutJob,
    util::cache::{ComputerMut, FrameCache},
    Color32, Context, FontFamily, FontId, Layout, Response, TextEdit, TextFormat, Ui, Widget,
};

use crate::git::Diff;

pub struct CodeWidget {
    diff: Diff,
    range: Range<usize>,
}

impl CodeWidget {
    pub fn new(diff: Diff, range: Range<usize>) -> CodeWidget {
        CodeWidget { diff, range }
    }
}
impl Widget for CodeWidget {
    fn ui(self, ui: &mut Ui) -> Response {
        puffin::profile_function!("CodeWidget");
        let mut layouter = |ui: &egui::Ui, string: &str, _wrap_width: f32| {
            let layout_job: egui::text::LayoutJob = highlight(
                ui.ctx(),
                string,
                &self.diff.header_indices,
                &self.diff.insertion_indices,
                &self.diff.deletion_indices,
                &self.diff.neutral_indices,
            );
            ui.fonts(|f| f.layout_job(layout_job))
        };

        let Range { start, end } = self.range;
        let end = std::cmp::min(end, self.diff.lines.len());

        let test = self.diff.content.lines().collect::<Vec<&str>>();
        let content = &test[start..end].join("\n");

        ui.with_layout(Layout::left_to_right(egui::Align::Min), |ui| {
            puffin::profile_function!("ui.with_layout");
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
}

type HighlightCache = FrameCache<LayoutJob, LayoutHandler>;

fn highlight(
    ctx: &Context,
    text: &str,
    header_indices: &Vec<usize>,
    insertion_indices: &Vec<usize>,
    deletion_indices: &Vec<usize>,
    neutral_indices: &Vec<usize>,
) -> LayoutJob {
    impl ComputerMut<(&str, &Vec<usize>, &Vec<usize>, &Vec<usize>, &Vec<usize>), LayoutJob>
        for LayoutHandler
    {
        fn compute(
            &mut self,
            (text, header_indices, insertion_indices, deletion_indices, neutral_indices): (
                &str,
                &Vec<usize>,
                &Vec<usize>,
                &Vec<usize>,
                &Vec<usize>,
            ),
        ) -> LayoutJob {
            puffin::profile_function!();
            LayoutHandler::layout_job(
                text,
                header_indices,
                insertion_indices,
                deletion_indices,
                neutral_indices,
            )
        }
    }

    ctx.memory_mut(|mem| {
        mem.caches.cache::<HighlightCache>().get((
            text,
            header_indices,
            insertion_indices,
            deletion_indices,
            neutral_indices,
        ))
    })
}

#[derive(Debug, Default)]
struct LayoutHandler {}

impl LayoutHandler {
    fn layout_job(
        text: &str,
        header_indices: &[usize],
        insertion_indices: &[usize],
        deletion_indices: &[usize],
        neutral_indices: &[usize],
    ) -> LayoutJob {
        puffin::profile_function!();
        let mut job = LayoutJob::default();
        job.wrap.max_width = f32::INFINITY;

        let header_format = TextFormat::simple(
            FontId::new(12.0, FontFamily::Monospace),
            Color32::from_rgb(7, 138, 171),
        );
        let insertion_format =
            TextFormat::simple(FontId::new(12.0, FontFamily::Monospace), Color32::GREEN);
        let deletion_format =
            TextFormat::simple(FontId::new(12.0, FontFamily::Monospace), Color32::RED);
        let neutral_format =
            TextFormat::simple(FontId::new(12.0, FontFamily::Monospace), Color32::WHITE);

        for (i, line) in text.split('\n').enumerate() {
            if header_indices.contains(&i) {
                let green_part = line.split(' ').take(4).collect::<Vec<&str>>().join(" ");
                let white_part = line.split(' ').skip(4).collect::<Vec<&str>>().join(" ");
                job.append(&green_part, 0.0, header_format.clone());
                job.append(" ", 0.0, neutral_format.clone());
                job.append(&white_part, 0.0, neutral_format.clone());
                job.append("\n", 0.0, neutral_format.clone());
            }
            if insertion_indices.contains(&i) {
                job.append(format!("{line}\n").as_str(), 0.0, insertion_format.clone());
            }
            if deletion_indices.contains(&i) {
                job.append(format!("{line}\n").as_str(), 0.0, deletion_format.clone());
            }
            if neutral_indices.contains(&i) {
                job.append(format!("{line}\n").as_str(), 0.0, neutral_format.clone());
            }
        }

        job
    }
}
