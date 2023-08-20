use std::{env, ops::Range, path::PathBuf, sync::mpsc::Sender};

use egui::{Align, Color32, Context, Layout, Response, RichText, ScrollArea, Ui, Widget, Window};

use crate::{
    data::{DiffData, DiffType, Message},
    git::{Diff, Header, Line},
    ui::{
        code::CodeWidget, diff_type::DiffTypeSelection, origins::OriginsWidget, stats::StatsWidget,
    },
    AppData, ControlData,
};

mod code;
mod diff_type;
mod origins;
mod stats;

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

            ui.add(DiffTypeSelection::new(
                sender.clone(),
                &mut control_data.diff_type.clone(),
            ));

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
