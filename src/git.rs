use core::fmt;
use egui::{Color32, Label, RichText};
use git2::{DiffStats, Repository};
use std::{cell::RefCell, path::PathBuf, rc::Rc};

#[derive(Debug, Clone)]
pub struct Diff {
    old_file: PathBuf,
    new_file: PathBuf,
    pub headers: Vec<Header>,
    pub lines: Vec<Line>,
}

impl Diff {
    fn new(old_file: PathBuf, new_file: PathBuf, headers: Vec<Header>, lines: Vec<Line>) -> Diff {
        Diff {
            old_file,
            new_file,
            headers,
            lines,
        }
    }

    pub fn file_name(&self) -> String {
        self.old_file
            .to_str()
            .unwrap_or("Error fetching file name")
            .to_owned()
    }
}

impl fmt::Display for Diff {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "diff --git a/{} b/{}",
            self.old_file.to_str().unwrap_or("Error fetching file name"),
            self.new_file.to_str().unwrap_or("Error fetching file name"),
        )?;

        for line in &self.lines {
            write!(f, "{}", line)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Header {
    content: String,
    pub line: u32,
}

#[derive(Debug)]
struct HeaderParserError;

impl Header {
    fn new(raw: String) -> Result<Header, HeaderParserError> {
        let line: u32 = raw
            .split(' ')
            .nth(2)
            .ok_or(HeaderParserError)?
            .split(',')
            .next()
            .ok_or(HeaderParserError)?
            .get(1..)
            .ok_or(HeaderParserError)?
            .parse()
            .map_err(|_| HeaderParserError)?;

        Ok(Header { content: raw, line })
    }

    pub fn to_labels(&self) -> (Label, Label) {
        let green_part = self
            .content
            .split(' ')
            .take(4)
            .collect::<Vec<&str>>()
            .join(" ");
        let white_part = self
            .content
            .split(' ')
            .skip(4)
            .collect::<Vec<&str>>()
            .join(" ");

        let green_label = Label::new(
            RichText::new(green_part)
                .color(Color32::from_rgb(7, 138, 171))
                .monospace(),
        );
        let white_label = Label::new(RichText::new(white_part).color(Color32::WHITE).monospace());

        (green_label, white_label)
    }
}

#[derive(Debug, Clone)]
pub struct Line {
    pub old_lineno: Option<u32>,
    pub new_lineno: Option<u32>,
    pub content: String,
    pub origin: char,
}

impl Line {
    fn new(
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

    pub fn to_richtext(&self) -> RichText {
        RichText::new(self.to_string())
            .monospace()
            .color(self.color())
    }

    fn color(&self) -> Color32 {
        match self.origin {
            '+' => Color32::GREEN,
            '-' => Color32::RED,
            _ => Color32::WHITE,
        }
    }
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.origin, self.content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let (diffs, _) = get_diffs(".".to_owned()).unwrap();
        for diff in diffs {
            println!("{:#?}", diff);
        }
    }

    #[test]
    fn parse_header() {
        let header =
            Header::new("@@ -209,6 +222,33 @@ impl fmt::Display for Diff {".to_string()).unwrap();
        assert_eq!(header.line, 222)
    }
}

#[derive(Debug)]
pub struct DiffParsingError;

pub fn get_diffs(path: String) -> Result<(Vec<Diff>, DiffStats), DiffParsingError> {
    let repo = Repository::open(path).map_err(|_| DiffParsingError)?;
    let diffs = repo
        .diff_index_to_workdir(None, None)
        .map_err(|_| DiffParsingError)?;

    let line_groups = Rc::new(RefCell::new(Vec::new()));
    diffs
        .foreach(
            &mut |_delta, _num| {
                line_groups.borrow_mut().push(Vec::new());
                true
            },
            None,
            None,
            Some(
                &mut |_delta, _hunk, _line| match std::str::from_utf8(_line.content()) {
                    Ok(c) => {
                        let mut content = c.to_string();
                        if content.ends_with('\n') {
                            content.pop();
                            if content.ends_with('\r') {
                                content.pop();
                            }
                        }

                        let line = Line::new(
                            _line.old_lineno(),
                            _line.new_lineno(),
                            content,
                            _line.origin(),
                        );

                        match line_groups.borrow_mut().last_mut() {
                            Some(last) => {
                                last.push(line);
                                true
                            }
                            None => false,
                        }
                    }
                    Err(_) => false,
                },
            ),
        )
        .map_err(|_| DiffParsingError)?;

    let header_groups = Rc::new(RefCell::new(Vec::new()));
    diffs
        .foreach(
            &mut |_delta, _num| {
                header_groups.borrow_mut().push(Vec::new());
                true
            },
            None,
            Some(&mut |_delta, _hunk| {
                let mut content = std::str::from_utf8(_hunk.header()).unwrap().to_string();
                if content.ends_with('\n') {
                    content.pop();
                    if content.ends_with('\r') {
                        content.pop();
                    }
                }

                match Header::new(content) {
                    Ok(header) => match header_groups.borrow_mut().last_mut() {
                        Some(last) => {
                            last.push(header);
                            true
                        }
                        None => false,
                    },
                    Err(_) => false,
                }
            }),
            None,
        )
        .map_err(|_| DiffParsingError)?;

    let mut result = Vec::new();
    diffs
        .foreach(
            &mut |_delta, _num| {
                let Some(old_file) = _delta.old_file().path() else {
                    return false;
                };

                let Some(new_file) = _delta.new_file().path() else {
                    return false;
                };
                let mut hg = header_groups.borrow_mut();
                let Some(headers) = hg.first() else {
                    return false;
                };

                let mut lg = line_groups.borrow_mut();
                let Some(lines) = lg.first() else {
                    return false;
                };

                let diff = Diff::new(
                    old_file.to_path_buf(),
                    new_file.to_path_buf(),
                    headers.to_vec(),
                    lines.to_vec(),
                );
                result.push(diff);

                hg.remove(0);
                lg.remove(0);
                true
            },
            None,
            None,
            None,
        )
        .map_err(|_| DiffParsingError)?;

    Ok((result, diffs.stats().map_err(|_| DiffParsingError)?))
}
