use core::fmt;
use egui::{Color32, Label, RichText, ScrollArea, Ui};
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
    project_path: PathBuf,
    diffs: Vec<Diff>,
    stats: Option<DiffStats>,
    shown_diff: Option<Diff>,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.selection_area(ui);
            self.project_area(ui);
            self.diff_area(ui);
        });
    }
}

impl MyApp {
    fn selection_area(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading(RichText::new("Diff Viewer").color(Color32::WHITE));
            ui.separator();

            if ui
                .button(RichText::new("Open").color(Color32::WHITE))
                .clicked()
            {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    self.project_path = path.clone();
                    let (diffs, stats) = get_diffs(path.clone());
                    self.diffs = diffs;
                    self.stats = Some(stats);
                    self.shown_diff = self.diffs.first().cloned().or(None);
                }
            }

            if !self.diffs.is_empty()
                && ui
                    .button(RichText::new("Reset").color(Color32::WHITE))
                    .clicked()
            {
                self.diffs = Vec::new();
                self.shown_diff = None;
            }
        });
        ui.separator();
    }
    fn project_area(&mut self, ui: &mut Ui) {
        if !self.diffs.is_empty() {
            ui.heading(RichText::new(self.project_path.to_str().unwrap()).color(Color32::WHITE));
            ui.label(
                RichText::new(
                    self.stats
                        .as_ref()
                        .unwrap()
                        .to_buf(DiffStatsFormat::SHORT, 100)
                        .unwrap()
                        .as_str()
                        .unwrap(),
                )
                .color(Color32::WHITE),
            );

            for diff in &self.diffs {
                if self
                    .shown_diff
                    .as_ref()
                    .map_or(PathBuf::default(), |d| d.old_file.clone())
                    == diff.old_file
                {
                    if ui
                        .button(diff.old_file.to_str().unwrap())
                        .highlight()
                        .clicked()
                    {
                        self.shown_diff = Some(diff.clone());
                    }
                } else if ui.button(diff.old_file.to_str().unwrap()).clicked() {
                    self.shown_diff = Some(diff.clone());
                }
            }
            ui.separator();
        }
    }

    fn diff_area(&self, ui: &mut Ui) {
        if let Some(diff) = self.shown_diff.clone() {
            ScrollArea::vertical().show(ui, |ui| {
                ui.vertical(|ui| {
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

                        ui.horizontal(|ui| {
                            match line.origin {
                                '+' => ui.label(
                                    RichText::new(line.new_lineno.unwrap().to_string())
                                        .color(Color32::GRAY),
                                ),
                                '-' => ui.label(
                                    RichText::new(line.old_lineno.unwrap().to_string())
                                        .color(Color32::GRAY),
                                ),
                                _ => ui.label(
                                    RichText::new(line.new_lineno.unwrap().to_string())
                                        .color(Color32::GRAY),
                                ),
                            };
                            ui.label(line.to_richtext());
                        });
                    }
                })
            });
        }
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
    _status: DiffStatus,
    old_file: PathBuf,
    new_file: PathBuf,
    headers: Vec<Header>,
    lines: Vec<Line>,
}

impl Diff {
    fn new(
        _status: DiffStatus,
        old_file: PathBuf,
        new_file: PathBuf,
        headers: Vec<Header>,
        lines: Vec<Line>,
    ) -> Diff {
        Diff {
            _status,
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
        )
        .unwrap();

        for line in &self.lines {
            write!(f, "{}", line).unwrap();
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
        write!(f, "{}{}", self.origin, self.content)
    }
}

fn get_diffs(path: PathBuf) -> (Vec<Diff>, DiffStats) {
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
                    DiffStatus::from(_delta.status()),
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
        let (diffs, _) = get_diffs(PathBuf::from("."));
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
