use std::path::PathBuf;

use git2::Repository;
use gtk::gio::Cancellable;
use gtk::{glib, Application, ApplicationWindow, Button};
use gtk::{prelude::*, FileDialog};

const APP_ID: &str = "com.github.m4tthewde.Contrast";
const APP_TITLE: &str = "Contrast";

fn main() -> glib::ExitCode {
    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);

    // Run the application
    app.run()
}

fn build_ui(app: &Application) {
    // Create a button with label and margins
    let button = Button::builder()
        .label("Select repository")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let window = ApplicationWindow::builder()
        .application(app)
        .title(APP_TITLE)
        .child(&button)
        .build();

    // Connect to "clicked" signal of `button`
    button.connect_clicked(glib::clone!(@weak window => move |_| {
            let dialog = FileDialog::builder().title("Select repository").build();
            dialog.select_folder(Some(&window), Cancellable::NONE, move |folder| {
                if let Ok(folder) = folder {
                    let path = folder.path().expect("Error getting path");
                    let diffs = get_diffs(path);
                    println!("{:?}", diffs);
                 }
            })
        }
    ));

    // Create a window and set the title

    // Present window
    window.present();
}

fn get_diffs(path: PathBuf) {
    let repo = Repository::open(path).expect("Error opening repository");
    let diff = repo
        .diff_index_to_workdir(None, None)
        .expect("Error getting diff");

    diff.print(git2::DiffFormat::Patch, |delta, hunk, line| {
        let content = std::str::from_utf8(line.content()).unwrap();
        println!("{}", content);
        true
    })
    .unwrap();
}
