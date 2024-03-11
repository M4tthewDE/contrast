use anyhow::{anyhow, Result};
use flate2::read::ZlibDecoder;
use std::{
    fmt::Display,
    fs,
    io::{BufRead, Cursor, Read},
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

    let _commit = get_commit(repo, hash)?;
    todo!();
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

    let tree_hash = commit
        .split(" ")
        .nth(2)
        .map(|t| t.strip_suffix("\nparent"))
        .flatten()
        .ok_or(anyhow!("error parsing commit"))?;

    let bytes = get_object(repo, tree_hash)?;
    let entries = parse_tree(repo, &bytes)?;

    for entry in entries {
        println!("{}", entry);
    }

    Ok(())
}

fn get_object(repo: &PathBuf, hash: &str) -> Result<Vec<u8>> {
    let path = repo
        .join("objects")
        .join(hash[0..2].to_owned())
        .join(hash[2..].to_owned());
    let bytes = fs::read(path)?;
    let mut decoder = ZlibDecoder::new(Cursor::new(bytes));
    let mut bytes = Vec::new();
    decoder.read_to_end(&mut bytes)?;
    Ok(bytes)
}

const NUL: u8 = 0;
const SPACE: u8 = 32;

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
}

impl Display for TreeEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} {}    {}", self.mode, self.hash, self.name)?;
        for child in &self.children {
            write!(f, "   {}", child)?;
        }

        Ok(())
    }
}

fn parse_tree(repo: &PathBuf, bytes: &[u8]) -> Result<Vec<TreeEntry>> {
    let mut cursor = Cursor::new(bytes);
    let mut literal = [0u8; 4];
    cursor.read_exact(&mut literal)?;
    let literal = String::from_utf8(literal.to_vec())?;

    if literal == "blob" {
        let mut blob = Vec::new();
        cursor.read_to_end(&mut blob)?;
        return Ok(vec![TreeEntry::new(
            "".to_string(),
            "".to_string(),
            "".to_string(),
            Vec::new(),
            Some(blob),
        )]);
    }

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
        let hash: String = hash.iter().map(|b| format!("{:02x}", b)).collect();

        println!("parsing children for {}", name);
        let children = parse_tree(repo, &get_object(repo, &hash)?)?;
        let entry = TreeEntry::new(mode, name, hash, children, None);
        entries.push(entry);

        if cursor.position() == length {
            break;
        }
    }

    Ok(entries)
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
        let _commit = get_latest_commit(&PathBuf::from(".git")).unwrap();
    }

    /*
    040000 tree 28dd2a88014c934e00646eb9744fdad4126e5ccd    .github
    100644 blob ea8c4bf7f35f6f77f75d92ad8ce8349f6e81ddba    .gitignore
    100644 blob 306778b4c5833eb760e38e007fcdb3e1f04bb175    Cargo.lock
    100644 blob 92685634cfc62c8b7fb4eb65e54ae953ed6ce620    Cargo.toml
    100644 blob f288702d2fa16d3cdf0035b15a9fcbc552cd88e7    LICENSE
    100644 blob 7d7005a975cf30c19dd5f744bf8f1cf2731f0287    README.md
    040000 tree 1f19b49b2b4df317e7911b3a4ccff32982cbbd08    src
    040000 tree 798a3fa9e3a8fecb3c38ffa18beb07cb90295855    tests
        */

    /*
    40000 .github(*LNdntOn\100644 .gitignoreK_ow]4nݺ100644 Cargo.lock0gxŃ>`ͳKu100644 Cargo.tomlhV4,eJSl 100644 LICENSEp-/m<5ZR͈100644 README.md}pu0Ds40000 src+ML)40000 testsy?<8ː)XU
    */
}
