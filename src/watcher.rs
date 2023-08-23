use std::{
    path::{Path, PathBuf},
    sync::{mpsc::Sender, Arc, Mutex},
};

use ignore::{gitignore::Gitignore, Error};
use notify::{Event, RecursiveMode, Watcher};

use crate::data::Message;

pub struct WatcherError;

pub fn run_watcher(
    path: PathBuf,
    should_refresh: Arc<Mutex<bool>>,
    sender: Sender<Message>,
) -> Result<(), WatcherError> {
    puffin::profile_function!();

    let gitignore = get_gitignore(&path).map_err(|_| WatcherError {})?;

    let s = sender.clone();
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| match res {
        Ok(event) => {
            for p in &event.paths {
                if !gitignore.matched_path_or_any_parents(p, false).is_ignore() {
                    match should_refresh.lock() {
                        Ok(mut sr) => *sr = true,
                        Err(_) => {
                            s.send(Message::ShowError("Error acquiring mutex".to_string()))
                                .expect("");
                        }
                    }
                }
            }
        }
        Err(e) => println!("watch error: {:?}", e),
    })
    .map_err(|_| WatcherError {})?;

    watcher
        .watch(&path, RecursiveMode::Recursive)
        .map_err(|_| WatcherError {})?;

    sender
        .send(Message::UpdateWatcher(watcher))
        .expect("Channel closed unexpectedly!");

    Ok(())
}
fn get_gitignore(path: &Path) -> Result<Gitignore, Error> {
    puffin::profile_function!();
    let (gitignore, error) = Gitignore::new(path.join(".gitignore"));
    if let Some(e) = error {
        return Err(e);
    }
    Ok(gitignore)
}
