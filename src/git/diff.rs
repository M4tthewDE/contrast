// https://blog.jcoglan.com/2017/02/12/the-myers-diff-algorithm-part-1/

use std::io::{BufRead, Cursor, Read};

use anyhow::{anyhow, Context, Result};
use chrono::NaiveDateTime;

#[derive(Debug, Clone)]
enum Version {
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
struct IndexFile {
    version: Version,
    index_entries: Vec<IndexEntry>,
}

// https://git-scm.com/docs/index-format
fn parse_index_file(bytes: &[u8]) -> Result<IndexFile> {
    let mut cursor = Cursor::new(bytes);
    let mut signature = [0u8; 4];
    cursor.read_exact(&mut signature)?;
    if signature != [68, 73, 82, 67] {
        return Err(anyhow!("Invalid signature: {:?}", signature));
    }

    let mut version = [0u8; 4];
    cursor.read_exact(&mut version)?;
    let version = Version::try_from(u32::from_be_bytes(version))?;

    assert!(matches!(version, Version::Two), "only supports version 2");

    let mut index_entry_num = [0u8; 4];
    cursor.read_exact(&mut index_entry_num)?;
    let index_entry_num = u32::from_be_bytes(index_entry_num);

    let mut index_entries = Vec::new();
    for _ in 0..index_entry_num {
        let index_entry = parse_index_entry(&mut cursor, &version)?;
        index_entries.push(index_entry);
    }

    Ok(IndexFile {
        version,
        index_entries,
    })
}

#[derive(Debug)]
enum ModeType {
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
struct IndexEntry {
    metadata_changed: NaiveDateTime,
    data_changed: NaiveDateTime,
    dev: u32,
    ino: u32,
    mode_type: ModeType,
    user_permissions: u32,
    group_permissions: u32,
    other_permissions: u32,
    uid: u32,
    gid: u32,
    file_size: u32,
    assume_valid: bool,
    extended: bool,
    stage: u16,
    name_length: u16,
    name: String,
}

fn parse_index_entry(cursor: &mut Cursor<&[u8]>, version: &Version) -> Result<IndexEntry> {
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

    let mut flags = [0u8; 2];
    cursor.read_exact(&mut flags)?;
    let flags = u16::from_be_bytes(flags);

    let assume_valid = flags >> 15 != 0;

    let extended = flags >> 14 != 0;
    if matches!(version, Version::Two) {
        assert_eq!(extended, false)
    }

    let stage = (flags >> 13) & 12;
    let name_length = flags & 4095;

    let mut name = Vec::new();
    let read_name_length = cursor.read_until(0u8, &mut name)?;
    assert_eq!(read_name_length - 1, name_length.into());
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
        assume_valid,
        extended,
        stage,
        name_length,
        name,
    })
}

#[cfg(test)]
mod tests {
    use crate::git::diff::{ModeType, Version};

    use super::parse_index_file;

    #[test]
    fn test_parse_index() {
        let bytes = include_bytes!("../../tests/data/index");
        let index_file = parse_index_file(bytes).unwrap();

        assert!(matches!(index_file.version, Version::Two));
        assert_eq!(index_file.index_entries.len(), 26);
        dbg!(&index_file.index_entries[0]);
        assert_eq!(index_file.index_entries[0].dev, 2051);
        assert_eq!(index_file.index_entries[0].ino, 15886105);
        assert!(matches!(
            index_file.index_entries[0].mode_type,
            ModeType::RegularFile
        ));
        assert_eq!(index_file.index_entries[0].user_permissions, 6);
        assert_eq!(index_file.index_entries[0].group_permissions, 4);
        assert_eq!(index_file.index_entries[0].other_permissions, 4);
        assert_eq!(index_file.index_entries[0].uid, 1000);
        assert_eq!(index_file.index_entries[0].gid, 1000);
        assert_eq!(index_file.index_entries[0].file_size, 388);
        assert_eq!(index_file.index_entries[0].assume_valid, false);
        assert_eq!(index_file.index_entries[0].extended, false);
        assert_eq!(index_file.index_entries[0].stage, 0);
        assert_eq!(index_file.index_entries[0].name_length, 26);
        assert_eq!(
            index_file.index_entries[0].name,
            ".github/workflows/rust.yml"
        );
    }
}
