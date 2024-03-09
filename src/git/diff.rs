use anyhow::Result;

fn calculate_diff(old: &[u8], new: &[u8]) -> Result<()> {
    todo!("implement");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::calculate_diff;

    #[test]
    fn test_calculate_diff() {
        let old = include_bytes!("../../tests/data/old.yml");
        let new = include_bytes!("../../tests/data/new.yml");

        let diff = calculate_diff(old, new).unwrap();
    }
}
