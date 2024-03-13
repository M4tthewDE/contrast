use std::{
    fs,
    io::{BufRead, Cursor, Read},
    path::Path,
};

use anyhow::{anyhow, Result};
use chrono::{DateTime, FixedOffset, NaiveDateTime};
use flate2::read::ZlibDecoder;

use crate::git::head;

#[derive(Debug, Clone)]
pub struct Commit {
    pub hash: String,
    pub tree: String,
    pub parents: Vec<String>,
    pub author: Author,
    pub commiter: Author,
    pub message: String,
}

impl Commit {
    fn new(repo: &Path, hash: &str) -> Result<Commit> {
        let commit_path = repo.join("objects").join(&hash[0..2]).join(&hash[2..]);

        let bytes = fs::read(commit_path)?;
        let mut decoder = ZlibDecoder::new(Cursor::new(bytes));
        let mut bytes = Vec::new();
        decoder.read_to_end(&mut bytes)?;
        let mut cursor = Cursor::new(bytes);

        let mut prefix = Vec::new();
        cursor.read_until(0, &mut prefix)?;
        prefix.remove(prefix.len() - 1);

        let prefix = String::from_utf8(prefix)?;
        if !prefix.starts_with("commit") {
            return Err(anyhow!("wrong prefix: {}", prefix));
        }

        let mut content = String::new();
        cursor.read_to_string(&mut content)?;
        let mut lines = content.lines();

        let tree = lines
            .next()
            .and_then(|l| l.split(' ').nth(1))
            .ok_or(anyhow!("no tree found"))?;

        let parents = lines
            .clone()
            .take_while(|l| l.starts_with("parent"))
            .map(|l| l.split(' ').nth(1))
            .filter_map(|x| x.map(|s| s.to_string()))
            .collect::<Vec<String>>();

        let mut lines = lines.skip(parents.len());
        let author = Author::new(
            &lines
                .next()
                .ok_or(anyhow!("no author found"))?
                .chars()
                .skip(7)
                .collect::<String>(),
        )?;
        let commiter = Author::new(
            &lines
                .next()
                .ok_or(anyhow!("no author found"))?
                .chars()
                .skip(10)
                .collect::<String>(),
        )?;
        let message = lines
            .skip_while(|l| !l.is_empty())
            .map(|l| l.to_string())
            .collect::<Vec<String>>()
            .join("\n");

        Ok(Commit {
            hash: hash.to_string(),
            tree: tree.to_string(),
            parents,
            author,
            commiter,
            message,
        })
    }

    pub fn contains(&self, search_string: &str) -> bool {
        let search_string = &search_string.to_lowercase();
        self.author.name.to_lowercase().contains(search_string)
            || self.message.to_lowercase().contains(search_string)
    }
}

#[derive(Debug, Clone)]
pub struct Author {
    pub name: String,
    pub timestamp: DateTime<FixedOffset>,
}

impl Author {
    fn new(line: &str) -> Result<Author> {
        let name = line.chars().take_while(|c| *c != '>').collect::<String>() + ">";
        let binding = line.chars().skip(name.len() + 1).collect::<String>();
        let mut parts = binding.split(' ');
        let timestamp = parts
            .next()
            .ok_or(anyhow!("no timestamp found"))?
            .parse::<i64>()?;
        let timestamp = NaiveDateTime::from_timestamp_opt(timestamp, 0)
            .ok_or(anyhow!("invalid timestamp: {}", timestamp))?;

        let offset = parts.next().ok_or(anyhow!("no offset found"))?;
        let offset_hours = offset[1..3].parse::<i32>().unwrap();
        let offset_minutes = offset[3..].parse::<i32>().unwrap();
        let offset = FixedOffset::east_opt(offset_hours * 3600 + offset_minutes * 60)
            .ok_or(anyhow!("invalid offset"))?;
        let timestamp: DateTime<FixedOffset> =
            DateTime::from_naive_utc_and_offset(timestamp, offset);

        Ok(Author { name, timestamp })
    }
}

pub fn get_log(repo: &Path) -> Result<Vec<Commit>> {
    let repo = repo.join(".git");
    let hash = head::get_hash(&repo)?;

    let mut commits = Vec::new();
    let mut parent = hash;
    loop {
        let commit = Commit::new(&repo, &parent).unwrap();
        commits.push(commit.clone());

        if let Some(p) = &commit.parents.first() {
            parent = p.to_string();
        } else {
            break;
        }
    }

    Ok(commits)
}
