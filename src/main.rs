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

fn get_diffs(path: PathBuf) {
    let repo = Repository::open(path).expect("Error opening repository");
    let diff = repo
        .diff_index_to_workdir(None, None)
        .expect("Error getting diff");

    diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
        let content = std::str::from_utf8(line.content()).unwrap();
        let origin = line.origin();

        match origin {
            '+' => print!("{origin}{content}"),
            '-' => print!("{origin}{content}"),
            _ => print!("{content}"),
        }

        true
    })
    .unwrap();
}
