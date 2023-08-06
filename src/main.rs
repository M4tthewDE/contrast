use git2::Repository;
use std::path::PathBuf;

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
    diff: Option<String>,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Open project...").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    self.diff = Some(get_diff(path.clone()));
                }
            }

            if let Some(diff) = &self.diff {
                ui.label(diff);
            }
        });
    }
}

fn get_diff(path: PathBuf) -> String {
    let repo = Repository::open(path).expect("Error opening repository");
    let diff = repo
        .diff_index_to_workdir(None, None)
        .expect("Error getting diff");

    let mut result = String::new();
    diff.foreach(&mut file_cb, Some(&mut binary_cb), Some(&mut hunk_cb), None)
        .unwrap();

    diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
        let content = std::str::from_utf8(line.content()).unwrap();
        let origin = line.origin();

        match origin {
            '+' => result.push_str(format!("{origin}{content}").as_str()),
            '-' => result.push_str(format!("{origin}{content}").as_str()),
            _ => result.push_str(format!("{content}").as_str()),
        }

        true
    })
    .unwrap();

    result
}

fn file_cb(delta: git2::DiffDelta, num: f32) -> bool {
    true
}

fn binary_cb(delta: git2::DiffDelta, binary: git2::DiffBinary) -> bool {
    true
}

// THE INFORMATION IS THERE, JUST NEED TO PARSE IT
fn hunk_cb(delta: git2::DiffDelta, binary: git2::DiffHunk) -> bool {
    let header = std::str::from_utf8(binary.header()).unwrap();
    println!("{:?}", delta.old_file());
    println!("{:?}", delta.new_file());
    println!("{header}");
    true
}
