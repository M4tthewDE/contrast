use std::fmt;

#[derive(Debug, Clone)]
pub struct Line {
    pub old_lineno: Option<u32>,
    pub new_lineno: Option<u32>,
    pub content: String,
    pub origin: char,
}

impl Line {
    pub fn new(
        old_lineno: Option<u32>,
        new_lineno: Option<u32>,
        content: String,
        origin: char,
    ) -> Line {
        Line {
            old_lineno,
            new_lineno,
            content,
            origin,
        }
    }
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.origin, self.content)
    }
}
