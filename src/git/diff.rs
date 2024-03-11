use std::{fmt::Display, fs, io, path::PathBuf};

use anyhow::{anyhow, Result};
use ignore::{Walk, WalkBuilder};

use crate::git::head;

use super::myers::Myers;

// Heavily inspired by:
// https://blog.jcoglan.com/2017/02/12/the-myers-diff-algorithm-part-1/

#[derive(Debug, Clone)]
pub struct Stats {
    pub files_changed: usize,
    pub total_insertions: usize,
    pub total_deletions: usize,
}

pub fn get_diffs(project_path: &PathBuf) -> Result<(Vec<Diff>, Stats)> {
    /*
    let mut files = Vec::new();
    for result in WalkBuilder::new(project_path).hidden(false).build() {
        let path = result?.into_path();
        if path.starts_with(project_path.join(PathBuf::from(".git"))) {
            continue;
        }
        files.push(path);
    }
    dbg!(files);
    */

    dbg!(&project_path);
    let commit = head::get_latest_commit(&project_path.join(".git/"))?;
    let blobs = commit.get_blobs(project_path.to_path_buf());

    let mut diffs = Vec::new();
    for (path, blob) in blobs {
        if let Ok(old) = String::from_utf8(blob) {
            let new = fs::read_to_string(path.clone())?;
            if let Some(diff) = calculate_diff(path, &old, &new)? {
                diffs.push(diff);
            }
        }
    }

    let files_changed = diffs.len();
    let mut total_insertions = 0;
    let mut total_deletions = 0;
    for diff in &diffs {
        total_insertions += diff.stats.insertions;
        total_deletions += diff.stats.deletions;
    }

    Ok((
        diffs,
        Stats {
            files_changed,
            total_insertions,
            total_deletions,
        },
    ))
}

#[derive(Debug, Clone)]
pub struct Diff {
    pub file_name: PathBuf,
    pub edits: Vec<DiffEdit>,
    pub stats: DiffStats,
}

// TODO: this should be Diff::new()
pub fn calculate_diff(file_name: PathBuf, a: &str, b: &str) -> Result<Option<Diff>> {
    if a == b {
        return Ok(None);
    }
    let a_lines = get_lines(a);
    let b_lines = get_lines(b);
    let edits = Myers::new(a_lines, b_lines).diff()?;
    let stats = DiffStats::new(&edits);

    Ok(Some(Diff {
        file_name,
        edits,
        stats,
    }))
}

fn get_lines(content: &str) -> Vec<DiffLine> {
    let mut lines = Vec::new();
    for (i, line) in content.lines().enumerate() {
        lines.push(DiffLine {
            number: i + 1,
            text: line.to_string(),
        })
    }

    lines
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiffLine {
    number: usize,
    pub text: String,
}

impl Display for DiffLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.number, self.text)
    }
}

#[derive(Debug, Clone)]
pub struct DiffStats {
    insertions: usize,
    deletions: usize,
}

impl DiffStats {
    fn new(edits: &Vec<DiffEdit>) -> DiffStats {
        let mut insertions = 0;
        let mut deletions = 0;

        for edit in edits {
            if matches!(edit.typ, EditType::Ins) {
                insertions += 1;
            }
            if matches!(edit.typ, EditType::Del) {
                deletions += 1;
            }
        }

        DiffStats {
            insertions,
            deletions,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum EditType {
    Ins,
    Del,
    Eql,
}

impl EditType {
    fn get_color(&self) -> String {
        match self {
            EditType::Ins => "\x1b[32m".to_string(),
            EditType::Del => "\x1b[31m".to_string(),
            EditType::Eql => "\x1b[39m".to_string(),
        }
    }

    fn get_tag(&self) -> String {
        match self {
            EditType::Ins => "+".to_string(),
            EditType::Del => "-".to_string(),
            EditType::Eql => " ".to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct DiffEdit {
    pub typ: EditType,
    pub a_line: Option<DiffLine>,
    pub b_line: Option<DiffLine>,
}

impl Display for DiffEdit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let color = self.typ.get_color();
        let reset = "\x1b[39m";
        let tag = self.typ.get_tag();
        let old_line = self
            .a_line
            .clone()
            .map(|a| a.number.to_string())
            .unwrap_or(" ".to_string());
        let new_line = self
            .b_line
            .clone()
            .map(|b| b.number.to_string())
            .unwrap_or(" ".to_string());
        let text = self
            .a_line
            .clone()
            .unwrap_or_else(|| self.b_line.clone().unwrap())
            .text;
        write!(f, "{color}{tag} {old_line} {new_line}   {text}{reset}",)
    }
}

impl DiffEdit {
    pub fn new(typ: EditType, a_line: Option<DiffLine>, b_line: Option<DiffLine>) -> DiffEdit {
        match typ {
            EditType::Ins => assert!(b_line.is_some()),
            EditType::Del => assert!(a_line.is_some()),
            EditType::Eql => assert!(a_line.is_some() && b_line.is_some()),
        }
        DiffEdit {
            typ,
            a_line,
            b_line,
        }
    }
}

#[cfg(test)]
mod tests {

    use std::path::PathBuf;

    use super::{calculate_diff, DiffEdit, DiffLine, EditType};

    #[test]
    fn test_calculate_diff() {
        let expected = vec![
            DiffEdit::new(
                EditType::Del,
                Some(DiffLine {
                    number: 1,
                    text: "A".to_string(),
                }),
                None,
            ),
            DiffEdit::new(
                EditType::Del,
                Some(DiffLine {
                    number: 2,
                    text: "B".to_string(),
                }),
                None,
            ),
            DiffEdit::new(
                EditType::Eql,
                Some(DiffLine {
                    number: 3,
                    text: "C".to_string(),
                }),
                Some(DiffLine {
                    number: 1,
                    text: "C".to_string(),
                }),
            ),
            DiffEdit::new(
                EditType::Ins,
                None,
                Some(DiffLine {
                    number: 2,
                    text: "B".to_string(),
                }),
            ),
            DiffEdit::new(
                EditType::Eql,
                Some(DiffLine {
                    number: 4,
                    text: "A".to_string(),
                }),
                Some(DiffLine {
                    number: 3,
                    text: "A".to_string(),
                }),
            ),
            DiffEdit::new(
                EditType::Eql,
                Some(DiffLine {
                    number: 5,
                    text: "B".to_string(),
                }),
                Some(DiffLine {
                    number: 4,
                    text: "B".to_string(),
                }),
            ),
            DiffEdit::new(
                EditType::Del,
                Some(DiffLine {
                    number: 6,
                    text: "B".to_string(),
                }),
                None,
            ),
            DiffEdit::new(
                EditType::Eql,
                Some(DiffLine {
                    number: 7,
                    text: "A".to_string(),
                }),
                Some(DiffLine {
                    number: 5,
                    text: "A".to_string(),
                }),
            ),
            DiffEdit::new(
                EditType::Ins,
                None,
                Some(DiffLine {
                    number: 6,
                    text: "C".to_string(),
                }),
            ),
        ];

        let old = include_str!("../../tests/data/a");
        let new = include_str!("../../tests/data/b");

        let diff = calculate_diff(PathBuf::from("tests/data/a"), old, new)
            .unwrap()
            .unwrap();

        assert_eq!(diff.edits.len(), expected.len());
        assert_eq!(diff.edits, expected);
        assert_eq!(diff.stats.insertions, 2);
        assert_eq!(diff.stats.deletions, 3);
    }
}
