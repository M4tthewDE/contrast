use crate::git;
use anyhow::{anyhow, Result};
use std::{
    collections::HashMap,
    fmt::{self, Display},
    fs,
    io::{BufRead, Cursor, Read},
    path::{Path, PathBuf},
};

use super::object;

const NUL: u8 = 0;
const SPACE: u8 = 32;

pub fn get_hash(repo: &Path) -> Result<String> {
    let content = fs::read_to_string(repo.join("HEAD"))?;
    let head = content
        .strip_prefix("ref: refs/heads/")
        .and_then(|h| h.strip_suffix('\n'))
        .map(|h| h.to_owned())
        .ok_or(anyhow!("error parsing HEAD"))?;

    let raw_hash = fs::read_to_string(repo.join("refs/heads").join(head.clone()))?;
    raw_hash
        .strip_suffix('\n')
        .map(|h| h.to_owned())
        .ok_or(anyhow!("error parsing refs/heads/{}", head))
}

#[derive(Debug)]
pub struct Head {
    tree: Vec<TreeEntry>,
}

impl Head {
    pub fn new(repo: &PathBuf) -> Result<Head> {
        let hash = get_hash(repo)?;
        let commit = object::get_string(repo, &hash)?;
        let commit_hash = commit
            .split(' ')
            .nth(2)
            .and_then(|t| t.strip_suffix("\nparent"))
            .ok_or(anyhow!("error parsing commit"))?;

        let tree = parse_tree(repo, &object::get_bytes(repo, commit_hash)?)?;

        Ok(Head { tree })
    }

    pub fn get_blobs(&self, path: PathBuf) -> HashMap<PathBuf, Vec<u8>> {
        let mut blobs = HashMap::new();
        for entry in &self.tree {
            blobs.extend(entry.get_blobs(path.clone()));
        }

        blobs
    }
}

#[derive(Debug)]
struct TreeEntry {
    mode: String,
    name: String,
    hash: String,
    children: Vec<TreeEntry>,
    blob: Option<Vec<u8>>,
}

impl TreeEntry {
    fn new(
        mode: String,
        name: String,
        hash: String,
        children: Vec<TreeEntry>,
        blob: Option<Vec<u8>>,
    ) -> TreeEntry {
        let mode = if mode.len() == 5 {
            "0".to_owned() + &mode
        } else {
            mode
        };
        TreeEntry {
            mode,
            name,
            hash,
            children,
            blob,
        }
    }

    fn get_blobs(&self, path: PathBuf) -> HashMap<PathBuf, Vec<u8>> {
        if self.children.is_empty() {
            let mut blob = HashMap::new();
            blob.insert(path.join(self.name.clone()), self.blob.clone().unwrap());
            return blob;
        }

        let mut blobs = HashMap::new();
        for child in &self.children {
            let child_blobs = child.get_blobs(path.join(self.name.clone()));
            blobs.extend(child_blobs);
        }

        blobs
    }
}

impl Display for TreeEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fn format_entry(entry: &TreeEntry, f: &mut fmt::Formatter, depth: usize) -> fmt::Result {
            let indent = "  ".repeat(depth);

            if entry.blob.is_some() {
                writeln!(
                    f,
                    "{}{} {} ({}) (blob data)",
                    indent, entry.mode, entry.name, entry.hash
                )?;
            } else {
                writeln!(
                    f,
                    "{}{} {} ({})",
                    indent, entry.mode, entry.name, entry.hash
                )?;
            }

            for child in &entry.children {
                format_entry(child, f, depth + 1)?;
            }

            Ok(())
        }

        format_entry(self, f, 0)
    }
}

fn parse_tree(repo: &PathBuf, bytes: &[u8]) -> Result<Vec<TreeEntry>> {
    let mut cursor = Cursor::new(bytes);

    let mut literal = [0u8; 4];
    cursor.read_exact(&mut literal)?;
    let literal = String::from_utf8(literal.to_vec())?;
    assert_eq!(literal, "tree");
    cursor.set_position(cursor.position() + 1);

    let mut length = Vec::new();
    cursor.read_until(NUL, &mut length)?;
    length.remove(length.len() - 1);
    let length_str = String::from_utf8(length)?;
    let length = length_str.parse::<u64>()? + length_str.len() as u64 + 6;

    let mut entries = Vec::new();
    loop {
        let mut mode = Vec::new();
        cursor.read_until(SPACE, &mut mode)?;
        mode.remove(mode.len() - 1);
        let mode = String::from_utf8(mode)?;

        let mut name = Vec::new();
        cursor.read_until(NUL, &mut name)?;
        name.remove(name.len() - 1);
        let name = String::from_utf8(name)?;

        let mut hash = [0u8; 20];
        cursor.read_exact(&mut hash)?;
        let hash = git::get_hash(&hash);

        if let Ok(blob) = git::parse_blob(object::get_bytes(repo, &hash)?) {
            let entry = TreeEntry::new(mode, name, hash, Vec::new(), Some(blob));
            entries.push(entry);
        } else {
            let children = parse_tree(repo, &object::get_bytes(repo, &hash)?)?;
            let entry = TreeEntry::new(mode, name, hash, children, None);
            entries.push(entry);
        }

        if cursor.position() == length {
            break;
        }
    }

    Ok(entries)
}
