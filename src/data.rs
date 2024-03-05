use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use notify::RecommendedWatcher;

use crate::git::{self, commit, commit::Commit, stats::Stats, Diff};

#[derive(Default)]
pub struct ControlData {
    pub show_err_dialog: bool,
    pub error_information: String,
    pub diff_type: DiffType,
    pub selected_diff: PathBuf,
    pub should_refresh: Arc<Mutex<bool>>,
    pub search_string: String,
    pub profiler: bool,
    pub log_open: bool,
}

#[derive(Clone)]
pub struct AppData {
    pub project_path: String,
    pub modified_diff_data: DiffData,
    pub staged_diff_data: DiffData,
    pub commits: Vec<Commit>,
}
#[derive(Clone)]
pub struct DiffData {
    pub diffs: Vec<Diff>,
    pub stats: Stats,
    pub file_tree: Tree,
}

impl DiffData {
    pub fn get_diff(&self, name: &PathBuf) -> Option<Diff> {
        for diff in &self.diffs {
            if diff.file_name() == *name {
                return Some(diff.clone());
            }
        }

        None
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
    Commits,
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
            diffs: modified_diffs.clone(),
            stats: modified_stats,
            file_tree: Tree::new(modified_diffs.iter().map(|d| d.file_name()).collect()),
        };

        let staged_diff_data = DiffData {
            diffs: staged_diffs.clone(),
            stats: staged_stats,
            file_tree: Tree::new(staged_diffs.iter().map(|d| d.file_name()).collect()),
        };

        let commits = commit::get_log(&project_path).map_err(|_| AppDataCreationError::Commits)?;

        Ok(AppData {
            project_path,
            modified_diff_data,
            staged_diff_data,
            commits,
        })
    }
}
pub enum Message {
    UpdateAppData(AppData),
    UpdateWatcher(RecommendedWatcher),
    ShowError(String),
}

#[derive(Debug, Clone)]
pub struct Tree {
    pub nodes: Vec<Tree>,
    pub files: Vec<File>,
    pub name: String,
    pub open: bool,
    pub id: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct File {
    pub path: PathBuf,
}

impl File {
    pub fn get_name(&self) -> Option<String> {
        Some(self.path.file_name()?.to_str()?.to_string())
    }
}

impl Tree {
    fn new(paths: Vec<PathBuf>) -> Self {
        let mut tree = Tree {
            nodes: vec![],
            files: vec![],
            name: "".to_owned(),
            open: true,
            id: 0,
        };

        for path in paths {
            tree.add(path, 0, 1);
        }

        tree
    }

    fn add(&mut self, path: PathBuf, depth: usize, id: u64) {
        // top level
        if path.components().count() == 1 {
            self.files.push(File { path });
            return;
        }

        // deepest level
        if path.components().count() == depth + 1 {
            self.files.push(File { path });
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
                node.add(path.clone(), depth + 1, id + 1);
                return;
            }
        }

        // create a new tree
        let mut tree = Tree {
            nodes: vec![],
            files: vec![],
            name,
            open: true,
            id,
        };
        tree.add(path, depth + 1, id + 1);
        self.nodes.push(tree);
    }

    pub fn toggle_open(&mut self, id: u64) {
        if self.id == id {
            self.open = !self.open;
            return;
        }

        for node in &mut self.nodes {
            node.toggle_open(id);
        }
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

        assert_eq!(tree.id, 0);
        assert_eq!(tree.name, "");
        assert_eq!(tree.open, true);

        assert_eq!(tree.nodes[0].id, 1);
        assert_eq!(tree.nodes[0].name, "src");
        assert_eq!(tree.nodes[0].open, true);
        assert_eq!(
            tree.nodes[0].files,
            vec![
                File {
                    path: PathBuf::from("src/data.rs")
                },
                File {
                    path: PathBuf::from("src/test.rs")
                }
            ]
        );
        assert_eq!(tree.nodes[0].nodes[0].id, 2);
        assert_eq!(tree.nodes[0].nodes[0].name, "ui");
        assert_eq!(tree.nodes[0].nodes[0].open, true);
        assert_eq!(
            tree.nodes[0].nodes[0].files,
            vec![File {
                path: PathBuf::from("src/ui/file_area.rs")
            },]
        );
    }
}
