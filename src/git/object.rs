use std::{
    fs,
    io::{Cursor, Read},
    path::Path,
};

use anyhow::{anyhow, Result};
use flate2::read::ZlibDecoder;

use crate::git;

pub fn get_bytes(repo: &Path, hash: &str) -> Result<Vec<u8>> {
    let path = repo.join("objects").join(&hash[0..2]).join(&hash[2..]);

    if path.exists() {
        let bytes = fs::read(path)?;
        let mut decoder = ZlibDecoder::new(Cursor::new(bytes));
        let mut content = Vec::new();
        decoder.read_to_end(&mut content)?;
        Ok(content)
    } else {
        let pack_dir = repo.join("objects/pack");
        if !pack_dir.exists() {
            Err(anyhow!("object not found {}", hash))
        } else {
            from_pack(&pack_dir, hash)
        }
    }
}

// https://git-scm.com/docs/pack-format
fn from_pack(dir: &Path, hash: &str) -> Result<Vec<u8>> {
    let index_files = IndexFile::from(dir)?;
    for index_file in index_files {
        dbg!(&index_file);
    }
    todo!();
}

const TOC: [u8; 4] = [255, 116, 79, 99];

#[derive(Debug)]
struct IndexFile {}

impl IndexFile {
    fn new(path: &Path) -> Result<IndexFile> {
        let bytes = fs::read(path)?;
        let byte_len = bytes.len() as u64;
        let mut cursor = Cursor::new(bytes);

        let mut toc = [0; 4];
        cursor.read_exact(&mut toc)?;
        if toc != TOC {
            return Err(anyhow!("invalid toc: {:?}", toc));
        }

        let mut version = [0; 4];
        cursor.read_exact(&mut version)?;
        let version = u32::from_be_bytes(version);
        if version != 2 {
            return Err(anyhow!("version {} is not supported", version));
        }

        let mut header = [0; 1024];
        cursor.read_exact(&mut header)?;

        loop {
            if cursor.position() == byte_len - 40 {
                break;
            }

            let mut offset = [0; 4];
            cursor.read_exact(&mut offset)?;
            let offset = u32::from_be_bytes(offset);
            dbg!(&offset);

            let mut hash = vec![0; 20];
            cursor.read_exact(&mut hash)?;
            let hash = git::get_hash(&hash);
            dbg!(hash);
        }
        todo!();
    }

    fn from(dir: &Path) -> Result<Vec<IndexFile>> {
        let mut index_files = Vec::new();
        for entry in dir.read_dir()? {
            let entry = entry?;
            if entry
                .file_name()
                .to_str()
                .map(|s| s.ends_with(".idx"))
                .unwrap_or_default()
            {
                index_files.push(IndexFile::new(&entry.path()).unwrap());
            }
        }

        Ok(index_files)
    }
}
