use anyhow::Result;
use chrono::NaiveDateTime;
use git2::{Repository, Sort};

#[derive(Debug, Clone)]
pub struct Commit {
    pub id: String,
    pub author: Author,
    pub message: String,
    pub time: NaiveDateTime,
}

impl Commit {
    pub fn contains(&self, search_string: &str) -> bool {
        let search_string = &search_string.to_lowercase();
        self.author.name.to_lowercase().contains(search_string)
            || self.author.email.to_lowercase().contains(search_string)
            || self.message.to_lowercase().contains(search_string)
    }
}

#[derive(Debug, Clone)]
pub struct Author {
    pub name: String,
    pub email: String,
}

pub fn get_log(path: &String) -> Result<Vec<Commit>> {
    let repo = Repository::open(path)?;
    let mut revwalk = repo.revwalk()?;
    revwalk.set_sorting(Sort::TIME)?;
    revwalk.push_head()?;

    let mut commits = Vec::new();

    for id in revwalk {
        let id = id?;
        let commit = repo.find_commit(id)?;

        let author = Author {
            name: commit.author().name().unwrap_or("").to_owned(),
            email: commit.author().email().unwrap_or("").to_owned(),
        };

        let commit = Commit {
            id: id.to_string(),
            author,
            message: commit.message().unwrap_or("").to_owned(),
            time: NaiveDateTime::from_timestamp_opt(
                commit.time().seconds() + commit.time().offset_minutes() as i64 * 60,
                0,
            )
            .unwrap_or_default(),
        };

        commits.push(commit);
    }

    Ok(commits)
}
