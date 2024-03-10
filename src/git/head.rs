use anyhow::{anyhow, Result};
use flate2::read::ZlibDecoder;
use std::{
    fs,
    io::{Cursor, Read},
    path::PathBuf,
};

fn get_head(repo: &PathBuf) -> Result<String> {
    let content = fs::read_to_string(repo.join("HEAD"))?;
    content
        .strip_prefix("ref: refs/heads/")
        .and_then(|h| h.strip_suffix("\n"))
        .map(|h| h.to_owned())
        .ok_or(anyhow!("error parsing HEAD"))
}

fn get_latest_commit(repo: &PathBuf) -> Result<()> {
    let head = get_head(repo)?;
    let raw_hash = fs::read_to_string(repo.join("refs/heads").join(head.clone()))?;
    let hash = raw_hash
        .strip_suffix("\n")
        .ok_or(anyhow!("error parsing refs/heads/{}", head))?;

    let commit = get_commit(repo, hash)?;
    todo!();
    Ok(())
}

fn get_commit(repo: &PathBuf, hash: &str) -> Result<()> {
    let commit_path = repo
        .join("objects")
        .join(hash[0..2].to_owned())
        .join(hash[2..].to_owned());

    let bytes = fs::read(commit_path)?;
    let mut decoder = ZlibDecoder::new(Cursor::new(bytes));
    let mut commit = String::new();
    decoder.read_to_string(&mut commit)?;
    println!("{}", commit);

    let tree_hash = commit
        .split(" ")
        .nth(2)
        .map(|t| t.strip_suffix("\nparent"))
        .flatten()
        .ok_or(anyhow!("error parsing commit"))?;
    dbg!(tree_hash);

    let tree_path = repo
        .join("objects")
        .join(tree_hash[0..2].to_owned())
        .join(tree_hash[2..].to_owned());

    let bytes = fs::read(tree_path)?;
    let mut decoder = ZlibDecoder::new(Cursor::new(bytes));
    let mut bytes = Vec::new();
    decoder.read_to_end(&mut bytes)?;
    println!("{}", bytes.len());

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::git::head::{get_head, get_latest_commit};

    #[test]
    fn test_get_head() {
        let head = get_head(&PathBuf::from(".git")).unwrap();
        assert_ne!(head, "");
    }

    #[test]
    fn test_get_latest_commit() {
        let commit = get_latest_commit(&PathBuf::from(".git")).unwrap();
    }
}
