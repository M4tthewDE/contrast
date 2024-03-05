use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Header {
    pub content: String,
    pub line: u32,
}

impl Header {
    pub fn new(raw: String) -> Result<Header> {
        let line: u32 = raw
            .split(' ')
            .nth(2)
            .context("less elements than expected")?
            .split(',')
            .next()
            .context("less elements than expected")?
            .get(1..)
            .context("less elements than expected")?
            .parse()?;

        Ok(Header { content: raw, line })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_header() {
        let header =
            Header::new("@@ -209,6 +222,33 @@ impl fmt::Display for Diff {".to_string()).unwrap();
        assert_eq!(header.line, 222)
    }
}
