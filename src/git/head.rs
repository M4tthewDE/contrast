use anyhow::{anyhow, Result};
use flate2::read::ZlibDecoder;
use std::{
    collections::HashMap,
    fmt::{self, Display},
    fs,
    io::{BufRead, Cursor, Read},
    path::{Path, PathBuf},
};

use crate::git::hash;

const NUL: u8 = 0;
const SPACE: u8 = 32;

fn get_head(repo: &Path) -> Result<String> {
    let content = fs::read_to_string(repo.join("HEAD"))?;
    content
        .strip_prefix("ref: refs/heads/")
        .and_then(|h| h.strip_suffix('\n'))
        .map(|h| h.to_owned())
        .ok_or(anyhow!("error parsing HEAD"))
}

#[derive(Debug)]
pub struct Commit {
    pub hash: String,
    tree: Vec<TreeEntry>,
}

impl Commit {
    pub fn get_blobs(&self, path: PathBuf) -> HashMap<PathBuf, Vec<u8>> {
        let mut blobs = HashMap::new();
        for entry in &self.tree {
            blobs.extend(entry.get_blobs(path.clone()));
        }

        blobs
    }
}

pub fn get_latest_commit(repo: &PathBuf) -> Result<Commit> {
    let head = get_head(repo)?;
    let raw_hash = fs::read_to_string(repo.join("refs/heads").join(head.clone()))?;
    let hash = raw_hash
        .strip_suffix('\n')
        .ok_or(anyhow!("error parsing refs/heads/{}", head))?;

    get_commit(repo, hash)
}

fn get_commit(repo: &PathBuf, hash: &str) -> Result<Commit> {
    let commit_path = repo.join("objects").join(&hash[0..2]).join(&hash[2..]);

    let bytes = fs::read(commit_path)?;
    let mut decoder = ZlibDecoder::new(Cursor::new(bytes));
    let mut commit = String::new();
    decoder.read_to_string(&mut commit)?;

    let commit_hash = commit
        .split(' ')
        .nth(2)
        .and_then(|t| t.strip_suffix("\nparent"))
        .ok_or(anyhow!("error parsing commit"))?;

    let tree = parse_tree(repo, &get_object(repo, commit_hash)?)?;

    Ok(Commit {
        hash: commit_hash.to_string(),
        tree,
    })
}

fn get_object(repo: &Path, hash: &str) -> Result<Vec<u8>> {
    let path = repo.join("objects").join(&hash[0..2]).join(&hash[2..]);
    let bytes = fs::read(path)?;
    let mut decoder = ZlibDecoder::new(Cursor::new(bytes));
    let mut bytes = Vec::new();
    decoder.read_to_end(&mut bytes)?;
    Ok(bytes)
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
        let hash = hash::from_bytes(&hash);

        if let Ok(blob) = parse_blob(get_object(repo, &hash)?) {
            let entry = TreeEntry::new(mode, name, hash, Vec::new(), Some(blob));
            entries.push(entry);
        } else {
            let children = parse_tree(repo, &get_object(repo, &hash)?)?;
            let entry = TreeEntry::new(mode, name, hash, children, None);
            entries.push(entry);
        }

        if cursor.position() == length {
            break;
        }
    }

    Ok(entries)
}

fn parse_blob(bytes: Vec<u8>) -> Result<Vec<u8>> {
    let mut cursor = Cursor::new(bytes);
    let mut literal = [0u8; 4];
    cursor.read_exact(&mut literal)?;
    let literal = String::from_utf8(literal.to_vec())?;

    if literal == "blob" {
        let mut trash = Vec::new();
        cursor.read_until(0, &mut trash)?;
        let mut blob = Vec::new();
        cursor.read_to_end(&mut blob)?;
        Ok(blob)
    } else {
        Err(anyhow!("not a blob"))
    }
}
