use std::path::PathBuf;

use crate::git::{self, Diff, Stats};

#[derive(Default)]
pub struct ControlData {
    pub show_err_dialog: bool,
    pub error_information: String,
    pub diff_type: DiffType,
}

#[derive(Clone)]
pub struct AppData {
    pub project_path: String,
    pub diffs: Vec<Diff>,
    pub stats: Stats,
    pub selected_diff_index: usize,
}

#[derive(PartialEq, Clone, Default)]
pub enum DiffType {
    #[default]
    Modified,
    Staged,
}

impl DiffType {
    pub fn label_text(&self) -> String {
        match self {
            DiffType::Modified => "Modified".to_string(),
            DiffType::Staged => "Staged".to_string(),
        }
    }
}

pub enum AppDataCreationError {
    Parsing,
}

impl AppData {
    pub fn from_pathbuf(path: PathBuf) -> Result<AppData, AppDataCreationError> {
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

    pub fn get_selected_diff(&self) -> Option<&Diff> {
        self.diffs.get(self.selected_diff_index)
    }
}

pub enum Message {
    LoadDiff(PathBuf),
    UpdateAppData(AppData),
    ShowError(String),
    ChangeDiffType(DiffType),
    CloseError,
}
