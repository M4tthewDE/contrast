use std::sync::mpsc::{self, Receiver, Sender};

use data::{AppData, ControlData};

use eframe::egui;
use egui::Context;

mod data;
mod git;
mod ui;

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        ..Default::default()
    };

    eframe::run_native("Contrast", options, Box::new(|_cc| Box::new(MyApp::new())))
}

struct MyApp {
    app_data: Option<AppData>,
    control_data: ControlData,
    sender: Sender<AppData>,
    receiver: Receiver<AppData>,
}

impl MyApp {
    fn new() -> MyApp {
        let (sender, receiver) = mpsc::channel();

        MyApp {
            app_data: None,
            control_data: ControlData::default(),
            sender,
            receiver,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        ui::show(
            ctx,
            &mut self.app_data,
            &mut self.control_data,
            &self.receiver,
            &self.sender,
        )
    }
}
