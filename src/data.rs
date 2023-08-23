use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use crate::git::{self, Diff, Stats};

#[derive(Default)]
pub struct ControlData {
    pub show_err_dialog: bool,
    pub error_information: String,
    pub diff_type: DiffType,
    pub selected_diff_index: usize,
    pub should_refresh: Arc<Mutex<bool>>,
}

#[derive(Clone)]
pub struct AppData {
    pub project_path: String,
    pub modified_diff_data: DiffData,
    pub staged_diff_data: DiffData,
}
#[derive(Clone)]
pub struct DiffData {
    pub diffs: Vec<Diff>,
    pub stats: Stats,
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

        let (modified_diffs, modified_stats) =
            git::get_diffs(&project_path).map_err(|_| AppDataCreationError::Parsing)?;

        let (staged_diffs, staged_stats) =
            git::get_staged_diffs(&project_path).map_err(|_| AppDataCreationError::Parsing)?;

        let modified_diff_data = DiffData {
            diffs: modified_diffs,
            stats: modified_stats,
        };

        let staged_diff_data = DiffData {
            diffs: staged_diffs,
            stats: staged_stats,
        };

        Ok(AppData {
            project_path,
            modified_diff_data,
            staged_diff_data,
        })
    }
}

pub enum Message {
    LoadDiff(PathBuf),
    UpdateAppData(AppData),
    ShowError(String),
    ChangeDiffType(DiffType),
    ChangeSelectedDiffIndex(usize),
    CloseError,
}
