// https://blog.jcoglan.com/2017/02/12/the-myers-diff-algorithm-part-1/

use std::slice::Iter;

use anyhow::{anyhow, Result};

fn parse_index_file(bytes: &[u8]) -> Result<()> {
    let signature = &bytes[0..4];
    if signature != &[68, 73, 82, 67] {
        return Err(anyhow!("Invalid signature: {:?}", signature));
    }

    let version = &bytes[7];
    if !vec![2, 3, 4].contains(version) {
        return Err(anyhow!("Invalid version: {:?}", version));
    }

    parse_index_entry(bytes[8..].iter())?;
    todo!();
}

fn parse_index_entry(bytes: Iter<u8>) -> Result<()> {
    let seconds = bytes
        .take(4)
        .copied()
        .collect::<Vec<u8>>()
        .try_into()
        .map(u32::from_be_bytes)
        .map_err(|_| anyhow!("Invalid seconds"))?;
    dbg!(seconds);
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
