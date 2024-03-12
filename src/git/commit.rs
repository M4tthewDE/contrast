use std::{
    fs,
    io::{BufRead, Cursor, Read},
    path::PathBuf,
};

use anyhow::{anyhow, Result};
use chrono::NaiveDateTime;
use flate2::read::ZlibDecoder;

use crate::git::head;

const SPACE: u8 = 32;
const NEWLINE: u8 = 10;

#[derive(Debug, Clone)]
pub struct Commit {
    pub id: String,
    pub author: Author,
    pub message: String,
    pub time: NaiveDateTime,
}

/*
[src/git/commit.rs:29:9] content = "commit 922\0tree 9ad6a8285cd9da8bea2ea26bb351399df9bceb3d\nparent d7cb04f94eed76644f092cfb54d33287560f575b\nauthor M4tthewDE <matthias.kronberg@gmail.com> 1710267252 +0100\ncommitter M4tthewDE <matthias.kronberg@gmail.com> 1710267252 +0100\ngpgsig
 -----BEGIN PGP SIGNATURE-----\n \n iQGzBAABCAAdFiEE+Cjgz2pfma5DAgbUeup303TONOAFAmXwm3gACgkQeup303TO\n NOC9rgv/TZ4ItbhB9By3BdWBTlDnmK/KGQo+eJA1p9PFTb5iYYPCyDUrZbhFV+eA\n CBY316qKcXh7ObUEefswZy4loVSB9ud8DoDX7/qplzLxROUSJDPJXo0ml7GNjgDL\n goqeUZlTz5iiFpJrJm4KW6Gm7pb8YCLHknjCRHBOTx9rUK
6jZSN4MPBHyGjpmdvI\n w860bN3PENmVCZjhYQYeXSmCNHYd+hSeeJ0oqq65jtRJRoQiO8YxKCQtBqFCq+3g\n mJ6l+rfYkl/0Cq9CuWDFtdL4cQoNv0DpLl7cAaOWIf3R3kaMSn64xXxrjluMVmCC\n hto+f7VdciUCztXuINW0xDAY31sn1jhTl0J4QirOjqvFO5hf8+VHlnpLnSSfPKYg\n B42v8/MPitZvd5GhB8TH1/oQOb3frzd9wjAFGmaBiO3Db2TF908GtrLV0PhrM
PA2\n 3hIp4A5xAXZ6/+CQXP3mhDcaTwF3kepe1XfNZVK040b+GSasIGtN7WcsPM1hMMzU\n p+Zglj5i\n =EkJI\n -----END PGP SIGNATURE-----\n\nSimplify head.rs\n"
*/

impl Commit {
    fn new(repo: &PathBuf, hash: &str) -> Result<Commit> {
        let commit_path = repo.join("objects").join(&hash[0..2]).join(&hash[2..]);

        let bytes = fs::read(commit_path)?;
        let mut decoder = ZlibDecoder::new(Cursor::new(bytes));
        let mut bytes = Vec::new();
        decoder.read_to_end(&mut bytes)?;
        let mut cursor = Cursor::new(bytes);

        let mut prefix = Vec::new();
        cursor.read_until(SPACE, &mut prefix)?;
        prefix.remove(prefix.len() - 1);

        let prefix = String::from_utf8(prefix)?;
        if prefix != "commit" {
            return Err(anyhow!("wrong prefix: {}", prefix));
        }

        let mut tree = Vec::new();
        cursor.read_until(NEWLINE, &mut tree)?;
        tree.remove(tree.len() - 1);

        let tree = String::from_utf8(tree)?;
        let tree = tree
            .split(' ')
            .nth(1)
            .ok_or(anyhow!("no tree hash found"))?;
        dbg!(&tree);

        let mut parent = Vec::new();
        cursor.read_until(NEWLINE, &mut parent)?;
        parent.remove(parent.len() - 1);

        let parent = String::from_utf8(parent)?;
        let parent = parent
            .split(' ')
            .nth(1)
            .ok_or(anyhow!("no parent hash found"))?;
        dbg!(&parent);

        let mut author = Vec::new();
        cursor.read_until(NEWLINE, &mut author)?;
        author.remove(author.len() - 1);
        let author = String::from_utf8(author)?;
        let author = Author::new(&author);
        dbg!(&author);

        todo!();
    }

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
    pub timestamp: NaiveDateTime,
}

impl Author {
    fn new(line: &str) -> Result<Author> {
        let mut parts = line.split(' ').skip(1);
        let name = parts.next().ok_or(anyhow!("no name found"))?.to_string();
        let email = parts.next().ok_or(anyhow!("no email found"))?;
        let email = email
            .get(1..email.len() - 1)
            .ok_or(anyhow!("invalid email format"))?
            .to_string();
        let timestamp = parts
            .next()
            .ok_or(anyhow!("no timestamp found"))?
            .parse::<i64>()?;
        let timestamp = NaiveDateTime::from_timestamp_opt(timestamp, 0)
            .ok_or(anyhow!("invalid timestamp: {}", timestamp))?;

        Ok(Author {
            name,
            email,
            timestamp,
        })
    }
}

pub fn get_log(repo: &PathBuf) -> Result<Vec<Commit>> {
    let repo = repo.join(".git");
    let hash = head::get_hash(&repo)?;

    let _commit = Commit::new(&repo, &hash).unwrap();

    todo!();
}
