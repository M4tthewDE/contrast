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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::git::head::get_head;

    #[test]
    fn test_get_head() {
        let head = get_head(&PathBuf::from(".git")).unwrap();
        assert_ne!(head, "");
    }
}
