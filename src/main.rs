use core::fmt;
use git2::{Delta, Repository};
use std::{cell::RefCell, ops::AddAssign, path::PathBuf, rc::Rc};

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
    picked_path: PathBuf,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Open project...").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    get_diffs(path.clone());
                    self.picked_path = path;
                }
            }
        });
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
struct Diff {
    status: DiffStatus,
    old_file: PathBuf,
    new_file: PathBuf,
    headers: Vec<String>,
    lines: Vec<Line>,
}

impl Diff {
    fn new(
        status: DiffStatus,
        old_file: PathBuf,
        new_file: PathBuf,
        headers: Vec<String>,
        lines: Vec<Line>,
    ) -> Diff {
        Diff {
            status,
            old_file,
            new_file,
            headers,
            lines,
        }
    }
}

impl fmt::Display for Diff {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "diff --git a/{} b/{}\n",
            self.old_file.to_str().unwrap(),
            self.new_file.to_str().unwrap(),
        )
        .unwrap();

        write!(f, "Status: {:?}\n", self.status).unwrap();

        for line in &self.lines {
            write!(f, "{}{}", line.origin, line.content).unwrap();
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
struct Line {
    content: String,
    origin: char,
}

impl Line {
    fn new(content: String, origin: char) -> Line {
        Line { content, origin }
    }
}

fn get_diffs(path: PathBuf) -> Vec<Diff> {
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
                let line = Line::new(
                    std::str::from_utf8(_line.content()).unwrap().to_string(),
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
                header_groups
                    .borrow_mut()
                    .last_mut()
                    .unwrap()
                    .push(std::str::from_utf8(_hunk.header()).unwrap().to_string());
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

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let diffs = get_diffs(PathBuf::from("."));
        for diff in diffs {
            println!("{}", diff);
        }
    }
}
