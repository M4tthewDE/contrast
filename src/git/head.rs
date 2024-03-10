use anyhow::{anyhow, Result};
use std::{fs, path::PathBuf};

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
    dbg!(hash);
    let commit_hash = fs::read_to_string(
        repo.join("objects")
            .join(hash[0..2].to_owned())
            .join(hash[2..].to_owned()),
    )?;

    dbg!(commit_hash.len());

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
