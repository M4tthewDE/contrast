use std::{path::PathBuf, sync::mpsc::Sender};

use egui::{ScrollArea, Ui};

use crate::data::{DiffData, Message};

pub fn ui(ui: &mut Ui, diff_data: &DiffData, index: usize, sender: &Sender<Message>) {
    puffin::profile_function!("files_area::ui");

    let mut index = index;
    ui.vertical(|ui| {
        ScrollArea::vertical()
            .id_source("file scroll area")
            .show(ui, |ui| {
                for (i, diff) in diff_data.diffs.iter().enumerate() {
                    if ui
                        .selectable_value(&mut index, i, diff.file_name().to_str().unwrap())
                        .clicked()
                    {
                        sender
                            .send(Message::ChangeSelectedDiffIndex(i))
                            .expect("Channel closed unexpectedly!");
                    }
                }
            });
    });
}

#[derive(Debug)]
struct Tree {
    nodes: Vec<Tree>,
    files: Vec<String>,
    name: String,
}

impl Tree {
    pub fn new(paths: Vec<PathBuf>) -> Self {
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

    pub fn add(&mut self, path: PathBuf, depth: usize) {
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
            PathBuf::from(r"src/main.rs"),
            PathBuf::from(r"src/test.rs"),
            PathBuf::from(r"src/asdf.rs"),
            PathBuf::from(r"src/module/mod.rs"),
            PathBuf::from(r"test.txt"),
        ];

        let tree = Tree::new(paths);

        dbg!(tree);
    }
}
