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

#[derive(Debug, Clone)]
struct Line {
    content: String,
}

impl Line {
    fn new(content: String) -> Line {
        Line { content }
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
                let line = Line::new(std::str::from_utf8(_line.content()).unwrap().to_string());
                line_groups.borrow_mut().last_mut().unwrap().push(line);
                true
            }),
        )
        .unwrap();

    let header_count = Rc::new(RefCell::new(Vec::new()));

    diffs
        .foreach(
            &mut |_delta, _num| {
                header_count.borrow_mut().push(0);
                true
            },
            None,
            Some(&mut |_delta, _hunk| {
                header_count.borrow_mut().last_mut().unwrap().add_assign(1);
                true
            }),
            None,
        )
        .unwrap();

    let headers = Rc::new(RefCell::new(Vec::new()));
    let mut result = Vec::new();
    diffs
        .foreach(
            &mut |_delta, _num| {
                println!("TEST");
                true
            },
            None,
            Some(&mut |_delta, _hunk| {
                headers
                    .borrow_mut()
                    .push(std::str::from_utf8(_hunk.header()).unwrap().to_string());

                if headers.borrow().len() == *header_count.borrow().first().unwrap() {
                    let diff = Diff::new(
                        DiffStatus::from(_delta.status()),
                        _delta.old_file().path().unwrap().to_path_buf(),
                        _delta.new_file().path().unwrap().to_path_buf(),
                        headers.borrow().clone(),
                        line_groups.borrow().first().unwrap().to_vec(),
                    );

                    result.push(diff);
                    headers.borrow_mut().clear();
                    header_count.borrow_mut().remove(0);
                    line_groups.borrow_mut().remove(0);
                }

                true
            }),
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
            println!("{:?}", diff);
        }
    }
}
