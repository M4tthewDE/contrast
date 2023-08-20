use std::{path::PathBuf, sync::mpsc::Sender};

use egui::{
    text::LayoutJob,
    util::cache::{ComputerMut, FrameCache},
    Align, Color32, ComboBox, Context, FontFamily, FontId, Layout, Response, RichText, ScrollArea,
    TextEdit, TextFormat, TextStyle, Ui, Widget, Window,
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
        if control_data.show_err_dialog {
            error_dialog(ctx, control_data, sender);
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

struct LineNumberWidget {
    max_digits: usize,
    line: Line,
}

impl LineNumberWidget {
    fn new(line: Line, max_digits: usize) -> LineNumberWidget {
        LineNumberWidget { line, max_digits }
    }
}

impl Widget for LineNumberWidget {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut line_no = match self.line.origin {
            '+' => self.line.new_lineno.unwrap_or(0).to_string(),
            '-' => self.line.old_lineno.unwrap_or(0).to_string(),
            _ => self.line.new_lineno.unwrap_or(0).to_string(),
        };

        while line_no.len() != self.max_digits {
            line_no = format!(" {}", line_no);
        }

        let line_no_richtext = RichText::new(line_no).color(Color32::GRAY).monospace();

        ui.label(line_no_richtext)
    }
}

struct LineNumbersWidget {
    longest_line: usize,
    lines: Vec<Line>,
    headers: Vec<Header>,
}

impl LineNumbersWidget {
    fn new(longest_line: usize, lines: Vec<Line>, headers: Vec<Header>) -> LineNumbersWidget {
        LineNumbersWidget {
            longest_line,
            lines,
            headers,
        }
    }
}

impl Widget for LineNumbersWidget {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.with_layout(Layout::top_down(egui::Align::Min), |ui| {
            ui.add_space(3.0);
            for line in &self.lines {
                for header in &self.headers {
                    if header.line == line.new_lineno.unwrap_or(0)
                        && line.origin != '+'
                        && line.origin != '-'
                    {
                        ui.label(RichText::new(" ").monospace());
                    }
                }
                ui.add(LineNumberWidget::new(line.clone(), self.longest_line));
            }
        })
        .response
    }
}

struct OriginsWidget {
    lines: Vec<Line>,
    headers: Vec<Header>,
}

impl OriginsWidget {
    fn new(lines: Vec<Line>, headers: Vec<Header>) -> OriginsWidget {
        OriginsWidget { lines, headers }
    }
}

impl Widget for OriginsWidget {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.with_layout(Layout::top_down(egui::Align::Min), |ui| {
            ui.add_space(3.0);
            for line in &self.lines {
                for header in &self.headers {
                    if header.line == line.new_lineno.unwrap_or(0)
                        && line.origin != '+'
                        && line.origin != '-'
                    {
                        ui.label(RichText::new(' ').monospace());
                    }
                }
                let line_color = match line.origin {
                    '+' => Color32::GREEN,
                    '-' => Color32::RED,
                    _ => Color32::WHITE,
                };

                ui.label(RichText::new(line.origin).color(line_color).monospace());
            }
        })
        .response
    }
}

struct CodeWidget {
    diff: Diff,
}

impl CodeWidget {
    fn new(diff: Diff) -> CodeWidget {
        CodeWidget { diff }
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

        ui.with_layout(Layout::left_to_right(egui::Align::Min), |ui| {
            ui.add(
                TextEdit::multiline(&mut self.diff.content.clone())
                    .font(TextStyle::Monospace)
                    .desired_width(f32::INFINITY)
                    .frame(false)
                    .code_editor()
                    .text_color(Color32::WHITE)
                    .lock_focus(true)
                    .layouter(&mut layouter),
            );
        })
        .response
    }
}

pub struct ProjectAreaWidget {
    path: String,
    stats: Stats,
}

impl ProjectAreaWidget {
    pub fn new(path: String, stats: Stats) -> ProjectAreaWidget {
        ProjectAreaWidget { path, stats }
    }
}

impl Widget for ProjectAreaWidget {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.heading(RichText::new(self.path.clone()).color(Color32::WHITE));
        ui.add(StatsWidget::new(self.stats));
        ui.separator()
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
        if self.diff.lines.is_empty() {
            return ui.label(RichText::new("No content").color(Color32::GRAY));
        }

        let longest_line = self.diff.get_longest_line();

        ui.vertical(|ui| {
            ScrollArea::both()
                .id_source("diff area")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.style_mut().spacing.item_spacing.y = 0.;
                        ui.add(LineNumbersWidget::new(
                            longest_line,
                            self.diff.lines.clone(),
                            self.diff.headers.clone(),
                        ));

                        ui.add(OriginsWidget::new(
                            self.diff.lines.clone(),
                            self.diff.headers.clone(),
                        ));

                        ui.add(CodeWidget::new(self.diff.clone()));
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
