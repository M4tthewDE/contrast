use core::fmt;
use egui::{Color32, Label, RichText, ScrollArea, Ui, Window};
use git2::{Delta, DiffStats, DiffStatsFormat, Repository};
use std::{cell::RefCell, path::PathBuf, rc::Rc};

use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        ..Default::default()
    };

    eframe::run_native("Contrast", options, Box::new(|_cc| Box::<MyApp>::default()))
}

#[derive(Default)]
struct MyApp {
    app_data: Option<AppData>,
    show_no_diff_dialog: bool,
}

struct AppData {
    project_path: String,
    diffs: Vec<Diff>,
    stats: DiffStats,
    selected_diff_index: usize,
}

impl AppData {
    fn new(path: PathBuf) -> Option<AppData> {
        let project_path = path.to_str()?.to_owned();
        let (diffs, stats) = get_diffs(project_path.clone());

        if diffs.is_empty() {
            return None;
        }

        Some(AppData {
            project_path,
            diffs,
            stats,
            selected_diff_index: 0,
        })
    }

    fn refresh(&mut self) {
        let (diffs, stats) = get_diffs(self.project_path.clone());
        self.diffs = diffs;
        self.stats = stats;
        self.selected_diff_index = 0;
    }

    fn get_selected_diff(&self) -> &Diff {
        // this can never fail (surely)
        self.diffs.get(self.selected_diff_index).unwrap()
    }

    fn get_stats_richtext(&self) -> RichText {
        // TODO: don't use to_buf and remove unwrap that way
        let stats = self.stats.to_buf(DiffStatsFormat::SHORT, 100).unwrap();
        let content = stats.as_str().unwrap_or("Error fetching stats");
        RichText::new(content).color(Color32::WHITE)
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.selection_area(ctx, ui);
            self.project_area(ui);

            if let Some(app_data) = &self.app_data {
                let diff = app_data.get_selected_diff();
                self.diff_area(ui, diff.clone());
            }
        });
    }
}

impl MyApp {
    fn selection_area(&mut self, ctx: &egui::Context, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading(RichText::new("Diff Viewer").color(Color32::WHITE));
            ui.separator();

            if ui
                .button(RichText::new("Open").color(Color32::WHITE))
                .clicked()
            {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    self.app_data = AppData::new(path);
                    if self.app_data.is_none() {
                        self.show_no_diff_dialog = true;
                    }
                }
            }

            if self.show_no_diff_dialog {
                Window::new("No diff found.")
                    .collapsible(false)
                    .resizable(true)
                    .show(ctx, |ui| {
                        if ui.button("Close").clicked() {
                            self.show_no_diff_dialog = false;
                        }
                    });
            }

            if self.app_data.is_some()
                && ui
                    .button(RichText::new("Refresh").color(Color32::WHITE))
                    .clicked()
            {
                if let Some(app_data) = &mut self.app_data {
                    app_data.refresh();
                }
            }
        });

        ui.separator();
    }

    fn project_area(&mut self, ui: &mut Ui) {
        if let Some(app_data) = &mut self.app_data {
            ui.heading(RichText::new(app_data.project_path.clone()).color(Color32::WHITE));
            ui.label(app_data.get_stats_richtext());

            for (i, diff) in app_data.diffs.iter().enumerate() {
                if app_data.selected_diff_index == i {
                    ui.button(diff.old_file.to_str().unwrap()).highlight();
                } else if ui.button(diff.old_file.to_str().unwrap()).clicked() {
                    app_data.selected_diff_index = i;
                }
            }
            ui.separator();
        }
    }

    fn diff_area(&self, ui: &mut Ui, diff: Diff) {
        let longest_line = self.get_longest_line(diff.clone());

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for line in &diff.lines {
                    for header in &diff.headers {
                        if header.line == line.new_lineno.unwrap_or(0)
                            && line.origin != '+'
                            && line.origin != '-'
                        {
                            let (green_label, white_label) = header.to_labels();
                            ui.horizontal(|ui| {
                                ui.add(green_label);
                                ui.add(white_label);
                            });
                        }
                    }

                    let line_no_richtext = self.get_line_no_richtext(line, longest_line);

                    ui.horizontal(|ui| {
                        ui.label(line_no_richtext);
                        ui.label(line.to_richtext());
                    });
                }
            });
    }

    fn get_line_no_richtext(&self, line: &Line, longest_line: u32) -> RichText {
        let mut line_no = match line.origin {
            '+' => line.new_lineno.unwrap().to_string(),
            '-' => line.old_lineno.unwrap().to_string(),
            _ => line.new_lineno.unwrap().to_string(),
        };

        while line_no.len() != longest_line.to_string().len() {
            line_no = format!(" {}", line_no);
        }

        RichText::new(line_no).color(Color32::GRAY).monospace()
    }

    fn get_longest_line(&self, diff: Diff) -> u32 {
        let mut longest_line = 0;
        for line in &diff.lines {
            let line_no = match line.origin {
                '+' => line.new_lineno.unwrap(),
                '-' => line.old_lineno.unwrap(),
                _ => line.new_lineno.unwrap(),
            };

            if line_no > longest_line {
                longest_line = line_no;
            }
        }

        longest_line
    }
}

#[derive(Debug, Clone)]
enum DiffStatus {
    Unmodified,
    Added,
    Deleted,
    Modified,
    Renamed,
    Copied,
    Ignored,
    Untracked,
    Typechange,
    Unreadable,
    Conflicted,
}

impl From<Delta> for DiffStatus {
    fn from(delta: Delta) -> Self {
        match delta {
            Delta::Unmodified => DiffStatus::Unmodified,
            Delta::Added => DiffStatus::Added,
            Delta::Deleted => DiffStatus::Deleted,
            Delta::Modified => DiffStatus::Modified,
            Delta::Renamed => DiffStatus::Renamed,
            Delta::Copied => DiffStatus::Copied,
            Delta::Ignored => DiffStatus::Ignored,
            Delta::Untracked => DiffStatus::Untracked,
            Delta::Typechange => DiffStatus::Typechange,
            Delta::Unreadable => DiffStatus::Unreadable,
            Delta::Conflicted => DiffStatus::Conflicted,
        }
    }
}

#[derive(Debug, Clone)]
struct Diff {
    old_file: PathBuf,
    new_file: PathBuf,
    headers: Vec<Header>,
    lines: Vec<Line>,
}

impl Diff {
    fn new(old_file: PathBuf, new_file: PathBuf, headers: Vec<Header>, lines: Vec<Line>) -> Diff {
        Diff {
            old_file,
            new_file,
            headers,
            lines,
        }
    }
}

impl fmt::Display for Diff {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "diff --git a/{} b/{}",
            self.old_file.to_str().unwrap(),
            self.new_file.to_str().unwrap(),
        )?;

        for line in &self.lines {
            write!(f, "{}", line)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
struct Header {
    content: String,
    line: u32,
}

impl Header {
    fn new(raw: String) -> Header {
        let line: u32 = raw
            .split(' ')
            .nth(2)
            .unwrap()
            .split(',')
            .next()
            .unwrap()
            .get(1..)
            .unwrap()
            .parse()
            .unwrap();
        Header { content: raw, line }
    }

    fn to_labels(&self) -> (Label, Label) {
        let green_part = self
            .content
            .split(' ')
            .take(4)
            .collect::<Vec<&str>>()
            .join(" ");
        let white_part = self
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

        (green_label, white_label)
    }
}

#[derive(Debug, Clone)]
struct Line {
    old_lineno: Option<u32>,
    new_lineno: Option<u32>,
    content: String,
    origin: char,
}

impl Line {
    fn new(
        old_lineno: Option<u32>,
        new_lineno: Option<u32>,
        content: String,
        origin: char,
    ) -> Line {
        Line {
            old_lineno,
            new_lineno,
            content,
            origin,
        }
    }

    fn to_richtext(&self) -> RichText {
        RichText::new(self.to_string())
            .monospace()
            .color(self.color())
    }

    fn color(&self) -> Color32 {
        match self.origin {
            '+' => Color32::GREEN,
            '-' => Color32::RED,
            _ => Color32::WHITE,
        }
    }
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.origin, self.content)
    }
}

fn get_diffs(path: String) -> (Vec<Diff>, DiffStats) {
    let repo = Repository::open(path).expect("Error opening repository");
    let diffs = repo
        .diff_index_to_workdir(None, None)
        .expect("Error getting diff");

    let line_groups = Rc::new(RefCell::new(Vec::new()));
    diffs
        .foreach(
            &mut |_delta, _num| {
                line_groups.borrow_mut().push(Vec::new());
                true
            },
            None,
            None,
            Some(&mut |_delta, _hunk, _line| {
                let mut content = std::str::from_utf8(_line.content()).unwrap().to_string();
                if content.ends_with('\n') {
                    content.pop();
                    if content.ends_with('\r') {
                        content.pop();
                    }
                }

                let line = Line::new(
                    _line.old_lineno(),
                    _line.new_lineno(),
                    content,
                    _line.origin(),
                );
                line_groups.borrow_mut().last_mut().unwrap().push(line);
                true
            }),
        )
        .unwrap();

    let header_groups = Rc::new(RefCell::new(Vec::new()));
    diffs
        .foreach(
            &mut |_delta, _num| {
                header_groups.borrow_mut().push(Vec::new());
                true
            },
            None,
            Some(&mut |_delta, _hunk| {
                let mut content = std::str::from_utf8(_hunk.header()).unwrap().to_string();
                if content.ends_with('\n') {
                    content.pop();
                    if content.ends_with('\r') {
                        content.pop();
                    }
                }

                header_groups
                    .borrow_mut()
                    .last_mut()
                    .unwrap()
                    .push(Header::new(content));

                true
            }),
            None,
        )
        .unwrap();

    let mut result = Vec::new();
    diffs
        .foreach(
            &mut |_delta, _num| {
                let diff = Diff::new(
                    _delta.old_file().path().unwrap().to_path_buf(),
                    _delta.new_file().path().unwrap().to_path_buf(),
                    header_groups.borrow().first().unwrap().to_vec(),
                    line_groups.borrow().first().unwrap().to_vec(),
                );
                result.push(diff);
                header_groups.borrow_mut().remove(0);
                line_groups.borrow_mut().remove(0);
                true
            },
            None,
            None,
            None,
        )
        .unwrap();

    (result, diffs.stats().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let (diffs, _) = get_diffs(".".to_owned());
        for diff in diffs {
            println!("{:#?}", diff);
        }
    }

    #[test]
    fn parse_header() {
        let header = Header::new("@@ -209,6 +222,33 @@ impl fmt::Display for Diff {".to_string());
        assert_eq!(header.line, 222)
    }
}
