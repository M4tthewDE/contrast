use anyhow::{anyhow, Result};
use flate2::read::ZlibDecoder;
use std::io::{BufRead, Read};
use std::{fs, io::Cursor, path::Path};

pub mod commit;
pub mod diff;
mod head;
mod index;
mod myers;

fn get_object(repo: &Path, hash: &str) -> Result<Vec<u8>> {
    let path = repo.join("objects").join(&hash[0..2]).join(&hash[2..]);
    let bytes = fs::read(path)?;
    let mut decoder = ZlibDecoder::new(Cursor::new(bytes));
    let mut bytes = Vec::new();
    decoder.read_to_end(&mut bytes)?;
    Ok(bytes)
}

pub fn get_hash(bytes: &[u8]) -> String {
    let hash: String = bytes.iter().fold(String::new(), |mut acc, b| {
        use std::fmt::Write; // Make sure to import `Write` at the top of your file
        write!(acc, "{:02x}", b).expect("Failed to write");
        acc
    });

    hash
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
