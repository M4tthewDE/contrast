use egui::{Color32, Label, Response, RichText, ScrollArea, Ui, Widget};

use crate::{
    git::{Diff, Header, Line, Stats},
    AppData,
};

struct LineWidget {
    max_digits: usize,
    line: Line,
}

impl LineWidget {
    fn new(line: Line, max_digits: usize) -> LineWidget {
        LineWidget { line, max_digits }
    }
}

impl Widget for LineWidget {
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
        let line_color = match self.line.origin {
            '+' => Color32::GREEN,
            '-' => Color32::RED,
            _ => Color32::WHITE,
        };
        let line_richtext = RichText::new(self.line.to_string())
            .monospace()
            .color(line_color);

        ui.horizontal(|ui| {
            ui.label(line_no_richtext);
            ui.label(line_richtext);
        })
        .response
    }
}

struct HeaderWidget {
    header: Header,
}

impl HeaderWidget {
    fn new(header: Header) -> HeaderWidget {
        HeaderWidget { header }
    }
}

impl Widget for HeaderWidget {
    fn ui(self, ui: &mut Ui) -> Response {
        let green_part = self
            .header
            .content
            .split(' ')
            .take(4)
            .collect::<Vec<&str>>()
            .join(" ");
        let white_part = self
            .header
            .content
            .split(' ')
            .skip(4)
            .collect::<Vec<&str>>()
            .join(" ");

        let green_label = Label::new(
            RichText::new(green_part)
                .color(Color32::from_rgb(7, 138, 171))
                .monospace(),
        );
        let white_label = Label::new(RichText::new(white_part).color(Color32::WHITE).monospace());

        ui.horizontal(|ui| {
            ui.add(green_label);
            ui.add(white_label);
        })
        .response
    }
}

pub struct ProjectAreaWidget {
    app_data: AppData,
}

impl ProjectAreaWidget {
    pub fn new(app_data: AppData) -> ProjectAreaWidget {
        ProjectAreaWidget { app_data }
    }
}

impl Widget for ProjectAreaWidget {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.heading(RichText::new(self.app_data.project_path.clone()).color(Color32::WHITE));
        ui.add(StatsWidget::new(self.app_data.stats));
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
                    for line in &self.diff.lines {
                        for header in &self.diff.headers {
                            if header.line == line.new_lineno.unwrap_or(0)
                                && line.origin != '+'
                                && line.origin != '-'
                            {
                                ui.add(HeaderWidget::new(header.clone()));
                            }
                        }

                        ui.add(LineWidget::new(line.clone(), longest_line));
                    }
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

        let content = format!(
            "{} file(s) changed, {} insertions(+), {} deletions(-)\n",
            file_changed_count, insertion_count, deletion_count
        );

        let richtext = RichText::new(content).color(Color32::WHITE);
        ui.label(richtext)
    }
}
