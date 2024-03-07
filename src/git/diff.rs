// https://blog.jcoglan.com/2017/02/12/the-myers-diff-algorithm-part-1/

use std::io::{Cursor, Read};

use anyhow::{anyhow, Result};

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

    let mut index_entry_num = [0u8; 4];
    cursor.read_exact(&mut index_entry_num)?;
    let index_entry_num = u32::from_be_bytes(index_entry_num);
    dbg!(index_entry_num);

    parse_index_entry(&mut cursor)?;
    todo!();
}

fn parse_index_entry(cursor: &mut Cursor<&[u8]>) -> Result<()> {
    let mut metadata_changed_secs = [0u8; 4];
    cursor.read_exact(&mut metadata_changed_secs)?;
    let metadata_changed_secs = u32::from_be_bytes(metadata_changed_secs);
    dbg!(metadata_changed_secs);
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
