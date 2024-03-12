use std::{
    fs,
    io::{BufRead, Cursor, Read},
    path::Path,
};

use anyhow::{anyhow, Context, Result};
use chrono::NaiveDateTime;

use crate::git;

#[derive(Debug, Clone)]
pub enum Version {
    Two,
    Three,
    Four,
}

impl TryFrom<u32> for Version {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> std::result::Result<Self, Self::Error> {
        match value {
            2 => Ok(Self::Two),
            3 => Ok(Self::Three),
            4 => Ok(Self::Four),
            _ => Err(anyhow!("Invalid version {value}")),
        }
    }
}

#[derive(Debug)]
pub struct IndexFile {
    pub version: Version,
    pub index_entries: Vec<IndexEntry>,
}

impl IndexFile {
    // https://git-scm.com/docs/index-format
    pub fn new(repo: &Path) -> Result<IndexFile> {
        let bytes = fs::read(repo.join(".git/index"))?;
        let mut cursor = Cursor::new(bytes);
        let mut signature = [0u8; 4];
        cursor.read_exact(&mut signature)?;
        if signature != [68, 73, 82, 67] {
            return Err(anyhow!("Invalid signature: {:?}", signature));
        }

        let mut version = [0u8; 4];
        cursor.read_exact(&mut version)?;
        let version = Version::try_from(u32::from_be_bytes(version))?;

        if !matches!(version, Version::Two) {
            return Err(anyhow!("Can't support version {:?} index file", version));
        }

        let mut index_entry_num = [0u8; 4];
        cursor.read_exact(&mut index_entry_num)?;
        let index_entry_num = u32::from_be_bytes(index_entry_num);

        let mut index_entries = Vec::new();
        for _ in 0..index_entry_num {
            let index_entry = IndexEntry::new(&mut cursor, repo)?;
            index_entries.push(index_entry);
        }

        Ok(IndexFile {
            version,
            index_entries,
        })
    }
}

#[derive(Debug)]
pub enum ModeType {
    RegularFile,
    SymbolicLink,
    GitLink,
}

impl TryFrom<u32> for ModeType {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> std::result::Result<Self, Self::Error> {
        match value {
            0b1000 => Ok(Self::RegularFile),
            0b1010 => Ok(Self::SymbolicLink),
            0b1110 => Ok(Self::GitLink),
            _ => Err(anyhow!("Invalid mode type {value}")),
        }
    }
}

#[derive(Debug)]
pub struct IndexEntry {
    pub metadata_changed: NaiveDateTime,
    pub data_changed: NaiveDateTime,
    pub dev: u32,
    pub ino: u32,
    pub mode_type: ModeType,
    pub user_permissions: u32,
    pub group_permissions: u32,
    pub other_permissions: u32,
    pub uid: u32,
    pub gid: u32,
    pub file_size: u32,
    pub hash: String,
    pub blob: Vec<u8>,
    pub assume_valid: bool,
    pub extended: bool,
    pub stage: u16,
    pub name_length: u16,
    pub name: String,
}

impl IndexEntry {
    fn new(cursor: &mut Cursor<Vec<u8>>, repo: &Path) -> Result<IndexEntry> {
        let mut metadata_changed_secs = [0u8; 4];
        cursor.read_exact(&mut metadata_changed_secs)?;
        let mut nanosec_fraction = [0u8; 4];
        cursor.read_exact(&mut nanosec_fraction)?;
        let metadata_changed = NaiveDateTime::from_timestamp_opt(
            u32::from_be_bytes(metadata_changed_secs) as i64,
            u32::from_be_bytes(nanosec_fraction),
        )
        .with_context(|| {
            format!(
                "Invalid timestamp {:?} {:?}",
                metadata_changed_secs, nanosec_fraction
            )
        })?;

        let mut data_changed_secs = [0u8; 4];
        cursor.read_exact(&mut data_changed_secs)?;
        let mut nanosec_fraction = [0u8; 4];
        cursor.read_exact(&mut nanosec_fraction)?;
        let data_changed = NaiveDateTime::from_timestamp_opt(
            u32::from_be_bytes(data_changed_secs) as i64,
            u32::from_be_bytes(nanosec_fraction),
        )
        .with_context(|| {
            format!(
                "Invalid timestamp {:?} {:?}",
                metadata_changed_secs, nanosec_fraction
            )
        })?;

        let mut dev = [0u8; 4];
        cursor.read_exact(&mut dev)?;
        let dev = u32::from_be_bytes(dev);

        let mut ino = [0u8; 4];
        cursor.read_exact(&mut ino)?;
        let ino = u32::from_be_bytes(ino);

        let mut mode = [0u8; 4];
        cursor.read_exact(&mut mode)?;
        let mode = u32::from_be_bytes(mode) & 65535;
        let mode_type = ModeType::try_from(mode >> 12)?;
        let user_permissions = (mode >> 6) & 7;
        let group_permissions = (mode >> 3) & 7;
        let other_permissions = mode & 7;

        let mut uid = [0u8; 4];
        cursor.read_exact(&mut uid)?;
        let uid = u32::from_be_bytes(uid);

        let mut gid = [0u8; 4];
        cursor.read_exact(&mut gid)?;
        let gid = u32::from_be_bytes(gid);

        let mut file_size = [0u8; 4];
        cursor.read_exact(&mut file_size)?;
        let file_size = u32::from_be_bytes(file_size);

        let mut hash = [0u8; 20];
        cursor.read_exact(&mut hash)?;
        let hash = git::get_hash(&hash);

        let blob = git::parse_blob(git::get_object(&repo.join(".git"), &hash).unwrap()).unwrap();

        let mut flags = [0u8; 2];
        cursor.read_exact(&mut flags)?;
        let flags = u16::from_be_bytes(flags);

        let assume_valid = flags >> 15 != 0;
        let extended = flags >> 14 != 0;
        let stage = (flags >> 13) & 12;
        let name_length = flags & 4095;

        let mut name = Vec::new();
        let read_name_length = cursor.read_until(0u8, &mut name)?;

        if read_name_length - 1 != name_length.into() {
            return Err(anyhow!(
                "Read name length does not match real name length: {} {}",
                read_name_length,
                name_length
            ));
        }

        name.remove(name.len() - 1);

        let name = String::from_utf8(name)?;
        loop {
            let mut buf = [0u8; 1];
            cursor.read_exact(&mut buf)?;
            if buf[0] != 0u8 {
                cursor.set_position(cursor.position() - 1);
                break;
            }
        }

        Ok(IndexEntry {
            metadata_changed,
            data_changed,
            dev,
            ino,
            mode_type,
            user_permissions,
            group_permissions,
            other_permissions,
            uid,
            gid,
            file_size,
            hash,
            blob,
            assume_valid,
            extended,
            stage,
            name_length,
            name,
        })
    }
}
