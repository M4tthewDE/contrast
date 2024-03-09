// https://blog.jcoglan.com/2017/02/12/the-myers-diff-algorithm-part-1/

use std::io::{Cursor, Read};

use anyhow::{anyhow, Result};
use chrono::{TimeZone, Utc};

// https://git-scm.com/docs/index-format
fn parse_index_file(bytes: &[u8]) -> Result<()> {
    let mut cursor = Cursor::new(bytes);
    let mut signature = [0u8; 4];
    cursor.read_exact(&mut signature)?;
    if signature != [68, 73, 82, 67] {
        return Err(anyhow!("Invalid signature: {:?}", signature));
    }

    let mut version = [0u8; 4];
    cursor.read_exact(&mut version)?;
    let version = u32::from_be_bytes(version);
    if ![2, 3, 4].contains(&version) {
        return Err(anyhow!("Invalid version: {:?}", version));
    }

    dbg!(version);

    let mut index_entry_num = [0u8; 4];
    cursor.read_exact(&mut index_entry_num)?;
    let index_entry_num = u32::from_be_bytes(index_entry_num);
    dbg!(index_entry_num);

    parse_index_entry(&mut cursor, version)?;
    todo!();
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

fn parse_index_entry(cursor: &mut Cursor<&[u8]>, version: u32) -> Result<()> {
    let mut metadata_changed_secs = [0u8; 4];
    cursor.read_exact(&mut metadata_changed_secs)?;
    let mut nanosec_fraction = [0u8; 4];
    cursor.read_exact(&mut nanosec_fraction)?;
    let metadata_changed = Utc.timestamp_opt(
        u32::from_be_bytes(metadata_changed_secs) as i64,
        u32::from_be_bytes(nanosec_fraction),
    );
    dbg!(metadata_changed);

    let mut data_changed_secs = [0u8; 4];
    cursor.read_exact(&mut data_changed_secs)?;
    let mut nanosec_fraction = [0u8; 4];
    cursor.read_exact(&mut nanosec_fraction)?;
    let data_changed = Utc.timestamp_opt(
        u32::from_be_bytes(data_changed_secs) as i64,
        u32::from_be_bytes(nanosec_fraction),
    );
    dbg!(data_changed);

    let mut dev = [0u8; 4];
    cursor.read_exact(&mut dev)?;
    let dev = u32::from_be_bytes(dev);
    dbg!(dev);

    let mut ino = [0u8; 4];
    cursor.read_exact(&mut ino)?;
    let ino = u32::from_be_bytes(ino);
    dbg!(ino);

    let mut mode = [0u8; 4];
    cursor.read_exact(&mut mode)?;
    let mode = u32::from_be_bytes(mode) & 65535;
    let mode_type = ModeType::try_from(mode >> 12)?;
    dbg!(mode_type);
    let permission = mode & 511;
    // FIXME: this permission value is not valid
    dbg!(permission);

    let mut uid = [0u8; 4];
    cursor.read_exact(&mut uid)?;
    let uid = u32::from_be_bytes(uid);
    dbg!(uid);

    let mut gid = [0u8; 4];
    cursor.read_exact(&mut gid)?;
    let gid = u32::from_be_bytes(gid);
    dbg!(gid);

    let mut file_size = [0u8; 4];
    cursor.read_exact(&mut file_size)?;
    let file_size = u32::from_be_bytes(file_size);
    dbg!(file_size);

    let mut hash = [0u8; 5];
    cursor.read_exact(&mut hash)?;

    let mut flags = [0u8; 2];
    cursor.read_exact(&mut flags)?;
    let flags = u16::from_be_bytes(flags);
    let assume_valid = flags >> 15 != 0;
    dbg!(assume_valid);

    let extended = (flags >> 14) & 2 != 0;
    dbg!(extended);
    if version == 2 {
        assert_eq!(extended, false)
    }

    let stage = (flags >> 13) & 12;
    dbg!(stage);

    let name_length = flags & 4095;
    dbg!(name_length);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::parse_index_file;

    #[test]
    fn test_parse_index() {
        let bytes = include_bytes!("../../tests/data/index");
        parse_index_file(bytes).unwrap();
    }
}
