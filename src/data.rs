use std::path::PathBuf;

use crate::git::{self, Diff, DiffParsingError, Stats};

#[derive(Default)]
pub struct ControlData {
    pub show_err_dialog: bool,
    pub error_information: String,
}

#[derive(Clone)]
pub struct AppData {
    pub project_path: String,
    pub diffs: Vec<Diff>,
    pub stats: Stats,
    pub selected_diff_index: usize,
}

pub enum AppDataCreationError {
    Parsing,
}

impl AppData {
    pub fn new(path: PathBuf) -> Result<AppData, AppDataCreationError> {
        let project_path = path
            .to_str()
            .ok_or(AppDataCreationError::Parsing)?
            .to_owned();
        let (diffs, stats) =
            git::get_diffs(project_path.clone()).map_err(|_| AppDataCreationError::Parsing)?;

        Ok(AppData {
            project_path,
            diffs,
            stats,
            selected_diff_index: 0,
        })
    }

    pub fn refresh(&mut self) -> Result<(), DiffParsingError> {
        let (diffs, stats) = git::get_diffs(self.project_path.clone())?;
        self.diffs = diffs;
        self.stats = stats;
        self.selected_diff_index = 0;

        Ok(())
    }

    pub fn get_selected_diff(&self) -> Option<&Diff> {
        self.diffs.get(self.selected_diff_index)
    }
}
