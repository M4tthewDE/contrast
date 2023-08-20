use std::{env, ops::Range, path::PathBuf, sync::mpsc::Sender};

use egui::{
    text::LayoutJob,
    util::cache::{ComputerMut, FrameCache},
    Align, Color32, ComboBox, Context, FontFamily, FontId, Layout, Response, RichText, ScrollArea,
    TextEdit, TextFormat, Ui, Widget, Window,
};

use crate::{
    data::{DiffData, DiffType, Message},
    git::{Diff, Header, Line, Stats},
    AppData, ControlData,
};

pub fn show(
    ctx: &Context,
    app_data: &Option<AppData>,
    control_data: &ControlData,
    sender: &Sender<Message>,
) {
    egui::CentralPanel::default().show(ctx, |ui| {
        puffin::profile_function!();
        if control_data.show_err_dialog {
            error_dialog(ctx, control_data, sender);
        }

        if env::var("PROFILING").is_ok() {
            puffin_egui::profiler_window(ctx);
        }

        ui.add(SelectionAreaWidget { app_data, sender });

        if let Some(app_data) = app_data {
            let diff_data = match control_data.diff_type {
                DiffType::Modified => &app_data.modified_diff_data,
                DiffType::Staged => &app_data.staged_diff_data,
            };

            ui.separator();
            ui.heading(RichText::new(app_data.project_path.clone()).color(Color32::WHITE));
            ui.separator();

            ui.add(DiffTypeSelectionArea {
                sender,
                selected_diff_type: &mut control_data.diff_type.clone(),
            });

            ui.add(StatsWidget::new(diff_data.stats.clone()));

            if diff_data.diffs.is_empty() {
                return;
            }

            ui.separator();

            ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                ui.add(FilesAreaWidget {
                    diff_data,
                    selected_diff_index: control_data.selected_diff_index,
                    sender,
                });
                ui.separator();

                if let Some(diff) = diff_data.diffs.get(control_data.selected_diff_index) {
                    ui.add(DiffAreaWidget::new(diff.clone()));
                }
            });
        }
    });
}

pub struct DiffTypeSelectionArea<'a> {
    sender: &'a Sender<Message>,
    selected_diff_type: &'a mut DiffType,
}

impl Widget for DiffTypeSelectionArea<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        puffin::profile_function!("DiffTypeSelectionArea");
        ui.horizontal(|ui| {
            ui.label("Type");
            ComboBox::from_id_source("Diff Type")
                .selected_text(self.selected_diff_type.label_text())
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_value(
                            self.selected_diff_type,
                            DiffType::Modified,
                            DiffType::Modified.label_text(),
                        )
                        .clicked()
                    {
                        self.sender
                            .send(Message::ChangeDiffType(DiffType::Modified))
                            .expect("Channel closed unexpectedly!");
                    };
                    if ui
                        .selectable_value(
                            self.selected_diff_type,
                            DiffType::Staged,
                            DiffType::Staged.label_text(),
                        )
                        .clicked()
                    {
                        self.sender
                            .send(Message::ChangeDiffType(DiffType::Staged))
                            .expect("Channel closed unexpectedly!");
                    };
                });
        })
        .response
    }
}

pub struct SelectionAreaWidget<'a> {
    app_data: &'a Option<AppData>,
    sender: &'a Sender<Message>,
}

impl Widget for SelectionAreaWidget<'_> {
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

pub fn error_dialog(ctx: &Context, control_data: &ControlData, sender: &Sender<Message>) {
    Window::new("Error")
        .collapsible(false)
        .resizable(true)
        .show(ctx, |ui| {
            ui.label(RichText::new(control_data.error_information.clone()).strong());
            if ui.button("Close").clicked() {
                sender
                    .send(Message::CloseError)
                    .expect("Channel closed unexpectedly!");
            }
        });
}

pub struct FilesAreaWidget<'a> {
    diff_data: &'a DiffData,
    selected_diff_index: usize,
    sender: &'a Sender<Message>,
}

impl Widget for FilesAreaWidget<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        puffin::profile_function!("FilesAreaWidget");
        ui.vertical(|ui| {
            ScrollArea::vertical()
                .id_source("file scroll area")
                .show(ui, |ui| {
                    for (i, diff) in self.diff_data.diffs.iter().enumerate() {
                        if self.selected_diff_index == i {
                            ui.button(diff.file_name()).highlight();
                        } else if ui.button(diff.file_name()).clicked() {
                            self.sender
                                .send(Message::ChangeSelectedDiffIndex(i))
                                .expect("Channel closed unexpectedly!");
                        }
                    }
                });
        })
        .response
    }
}

struct LineNumbersWidget {
    longest_line: usize,
    lines: Vec<Line>,
    headers: Vec<Header>,
    range: Range<usize>,
}

impl LineNumbersWidget {
    fn new(
        longest_line: usize,
        lines: Vec<Line>,
        headers: Vec<Header>,
        range: Range<usize>,
    ) -> LineNumbersWidget {
        LineNumbersWidget {
            longest_line,
            lines,
            headers,
            range,
        }
    }
}

impl Widget for LineNumbersWidget {
    fn ui(self, ui: &mut Ui) -> Response {
        puffin::profile_function!("LineNumbersWidget");

        let mut content = "".to_owned();

        let Range { start, end } = self.range;
        let end = std::cmp::min(end, self.lines.len());

        for line in &self.lines[start..end] {
            for header in &self.headers {
                if header.line == line.new_lineno.unwrap_or(0)
                    && line.origin != '+'
                    && line.origin != '-'
                {
                    content.push_str(" \n");
                }
            }
            let mut line_no = match line.origin {
                '+' => line.new_lineno.unwrap_or(0).to_string(),
                '-' => line.old_lineno.unwrap_or(0).to_string(),
                _ => line.new_lineno.unwrap_or(0).to_string(),
            };

            while line_no.len() != self.longest_line {
                line_no = format!(" {}", line_no);
            }

            content.push_str(format!("{}\n", line_no).as_str());
        }
        ui.vertical(|ui| {
            ui.add_space(3.0);
            ui.label(RichText::new(content).monospace().color(Color32::GRAY))
        })
        .response
    }
}

struct OriginsWidget {
    lines: Vec<Line>,
    headers: Vec<Header>,
    range: Range<usize>,
}

impl OriginsWidget {
    fn new(lines: Vec<Line>, headers: Vec<Header>, range: Range<usize>) -> OriginsWidget {
        OriginsWidget {
            lines,
            headers,
            range,
        }
    }
}

impl Widget for OriginsWidget {
    fn ui(self, ui: &mut Ui) -> Response {
        puffin::profile_function!("OriginsWidget");

        let Range { start, end } = self.range;
        let end = std::cmp::min(end, self.lines.len());

        let mut content = "".to_owned();
        for line in &self.lines[start..end] {
            for header in &self.headers {
                if header.line == line.new_lineno.unwrap_or(0)
                    && line.origin != '+'
                    && line.origin != '-'
                {
                    content.push_str(" \n");
                }
            }

            content.push_str(format!("{} \n", line.origin).as_str());
        }

        let mut layouter = |ui: &egui::Ui, string: &str, _wrap_width: f32| {
            let layout_job: egui::text::LayoutJob = origins_highlight(ui.ctx(), string);
            ui.fonts(|f| f.layout_job(layout_job))
        };
        ui.add(
            TextEdit::multiline(&mut content)
                .desired_width(0.0)
                .frame(false)
                .interactive(false)
                .layouter(&mut layouter),
        )
    }
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

struct CodeWidget {
    diff: Diff,
    range: Range<usize>,
}

impl CodeWidget {
    fn new(diff: Diff, range: Range<usize>) -> CodeWidget {
        CodeWidget { diff, range }
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

pub struct DiffAreaWidget {
    diff: Diff,
}

impl DiffAreaWidget {
    pub fn new(diff: Diff) -> DiffAreaWidget {
        DiffAreaWidget { diff }
    }
}

impl Widget for DiffAreaWidget {
    fn ui(self, ui: &mut Ui) -> Response {
        puffin::profile_function!("DiffAreaWidget");
        if self.diff.lines.is_empty() {
            return ui.label(RichText::new("No content").color(Color32::GRAY));
        }

        let longest_line = self.diff.get_longest_line();
        let total_rows = self.diff.lines.len() + self.diff.headers.len();

        ui.vertical(|ui| {
            ScrollArea::both()
                .id_source("diff area")
                .auto_shrink([false, false])
                .show_rows(ui, 10.0, total_rows, |ui, row_range| {
                    ui.horizontal(|ui| {
                        ui.add(LineNumbersWidget::new(
                            longest_line,
                            self.diff.lines.clone(),
                            self.diff.headers.clone(),
                            row_range.clone(),
                        ));
                        ui.add(OriginsWidget::new(
                            self.diff.lines.clone(),
                            self.diff.headers.clone(),
                            row_range.clone(),
                        ));
                        ui.add(CodeWidget::new(self.diff.clone(), row_range.clone()));
                    });
                });
        })
        .response
    }
}

pub struct StatsWidget {
    stats: Stats,
}

impl StatsWidget {
    pub fn new(stats: Stats) -> StatsWidget {
        StatsWidget { stats }
    }
}

impl Widget for StatsWidget {
    fn ui(self, ui: &mut Ui) -> Response {
        puffin::profile_function!("StatsWidget");
        let file_changed_count = self.stats.files_changed;
        let insertion_count = self.stats.insertions;
        let deletion_count = self.stats.deletions;

        let files_richtext = match file_changed_count {
            1 => {
                RichText::new(format!("{} file changed,", file_changed_count)).color(Color32::WHITE)
            }
            _ => RichText::new(format!("{} files changed,", file_changed_count))
                .color(Color32::WHITE),
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
        })
        .response
    }
}
