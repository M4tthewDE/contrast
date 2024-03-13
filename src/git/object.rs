use std::{
    fs,
    io::{Cursor, Read},
    path::Path,
};

use anyhow::{anyhow, Result};
use flate2::read::ZlibDecoder;

pub fn get_bytes(repo: &Path, hash: &str) -> Result<Vec<u8>> {
    let path = repo.join("objects").join(&hash[0..2]).join(&hash[2..]);

    if path.exists() {
        let bytes = fs::read(path)?;
        let mut decoder = ZlibDecoder::new(Cursor::new(bytes));
        let mut content = Vec::new();
        decoder.read_to_end(&mut content)?;
        Ok(content)
    } else {
        Err(anyhow!("object not found {}", hash))
    }
}

pub fn get_string(repo: &Path, hash: &str) -> Result<String> {
    let path = repo.join("objects").join(&hash[0..2]).join(&hash[2..]);

    if path.exists() {
        let bytes = fs::read(path)?;
        let mut decoder = ZlibDecoder::new(Cursor::new(bytes));
        let mut content = String::new();
        decoder.read_to_string(&mut content)?;
        Ok(content)
    } else {
        Err(anyhow!("object not found {}", hash))
    }
}
