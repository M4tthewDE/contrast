use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use notify::RecommendedWatcher;

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

impl DiffData {
    pub fn file_tree(&self) -> Tree {
        let paths = self.diffs.iter().map(|d| d.file_name()).collect();

        Tree::new(paths)
    }
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
    UpdateWatcher(RecommendedWatcher),
    ShowError(String),
    ChangeDiffType(DiffType),
    ChangeSelectedDiffIndex(usize),
    CloseError,
}

#[derive(Debug)]
pub struct Tree {
    pub nodes: Vec<Tree>,
    pub files: Vec<String>,
    pub name: String,
}

impl Tree {
    fn new(paths: Vec<PathBuf>) -> Self {
        let mut tree = Tree {
            nodes: vec![],
            files: vec![],
            name: "".to_owned(),
        };

        for path in paths {
            tree.add(path, 0);
        }

        tree
    }

    fn add(&mut self, path: PathBuf, depth: usize) {
        // base cases

        // top level
        if path.components().count() == 1 {
            self.files.push(path.to_str().unwrap().to_owned());
            return;
        }

        // deepest level
        if path.components().count() == depth {
            self.files
                .push(path.file_name().unwrap().to_str().unwrap().to_owned());
            return;
        }

        let name = path
            .components()
            .nth(depth)
            .unwrap()
            .as_os_str()
            .to_str()
            .unwrap()
            .to_owned();

        // do we already have a tree for this?
        for node in &mut self.nodes {
            if node.name == name {
                node.add(path.clone(), depth + 1);
                return;
            }
        }

        // create a new tree
        let mut tree = Tree {
            nodes: vec![],
            files: vec![],
            name,
        };
        tree.add(path, depth + 1);
        self.nodes.push(tree);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree() {
        let paths = vec![
            PathBuf::from(r"src/data.rs"),
            PathBuf::from(r"src/test.rs"),
            PathBuf::from(r"src/ui/file_area.rs"),
        ];

        let tree = Tree::new(paths);

        dbg!(tree);
    }
}
