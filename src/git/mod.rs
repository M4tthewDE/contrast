use anyhow::Result;
use core::fmt;
use git2::Repository;
use header::Header;
use line::Line;
use stats::Stats;
use std::{cell::RefCell, path::PathBuf, rc::Rc};

pub mod commit;
mod header;
mod line;
pub mod stats;

#[derive(Debug, Clone)]
pub struct Diff {
    old_file: PathBuf,
    new_file: PathBuf,
    pub headers: Vec<Header>,
    pub lines: Vec<Line>,
    pub content: String,
    pub origins_content: String,
    pub line_numbers: String,
    pub header_indices: Vec<usize>,
    pub insertion_indices: Vec<usize>,
    pub deletion_indices: Vec<usize>,
    pub neutral_indices: Vec<usize>,
}

impl Diff {
    fn new(old_file: PathBuf, new_file: PathBuf, headers: Vec<Header>, lines: Vec<Line>) -> Diff {
        let mut longest_line = 0;
        for line in &lines {
            let line_no = match line.origin {
                '+' => line.new_lineno.unwrap_or(0),
                '-' => line.old_lineno.unwrap_or(0),
                _ => line.new_lineno.unwrap_or(0),
            };

            if line_no > longest_line {
                longest_line = line_no;
            }
        }

        let longest_line_len = longest_line.to_string().len();

        let mut content = "".to_owned();
        let mut origins_content = "".to_owned();
        let mut lines_content = "".to_owned();
        let mut header_indices = Vec::new();
        let mut insertion_indices = Vec::new();
        let mut deletion_indices = Vec::new();
        let mut neutral_indices = Vec::new();

        let mut i = 0;
        for line in &lines {
            for header in &headers {
                if header.line == line.new_lineno.unwrap_or(0)
                    && line.origin != '+'
                    && line.origin != '-'
                {
                    content.push_str(format!("{}\n", header.content).as_str());
                    origins_content.push_str(" \n");
                    lines_content.push_str(" \n");
                    header_indices.push(i);
                    i += 1;
                }
            }
            let mut line_no = match line.origin {
                '+' => line.new_lineno.unwrap_or(0).to_string(),
                '-' => line.old_lineno.unwrap_or(0).to_string(),
                _ => line.new_lineno.unwrap_or(0).to_string(),
            };

            while line_no.len() != longest_line_len {
                line_no = format!(" {}", line_no);
            }

            content.push_str(format!("{}\n", line.content.as_str()).as_str());
            origins_content.push_str(format!("{} \n", line.origin).as_str());
            lines_content.push_str(format!("{}\n", line_no).as_str());

            match line.origin {
                '+' => insertion_indices.push(i),
                '-' => deletion_indices.push(i),
                _ => neutral_indices.push(i),
            };

            i += 1;
        }

        Diff {
            old_file,
            new_file,
            headers,
            lines,
            content,
            origins_content,
            line_numbers: lines_content,
            header_indices,
            insertion_indices,
            deletion_indices,
            neutral_indices,
        }
    }

    pub fn file_name(&self) -> PathBuf {
        self.old_file.to_owned()
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

pub fn get_staged_diffs(path: &String) -> Result<(Vec<Diff>, Stats)> {
    let repo = Repository::open(path)?;
    let head = repo.head()?.peel_to_tree()?;
    let diffs = repo.diff_tree_to_index(Some(&head), None, None)?;
    parse_diffs(diffs)
}

pub fn get_diffs(path: &String) -> Result<(Vec<Diff>, Stats)> {
    let repo = Repository::open(path)?;
    let diffs = repo.diff_index_to_workdir(None, None)?;
    parse_diffs(diffs)
}

fn parse_diffs(diffs: git2::Diff) -> Result<(Vec<Diff>, Stats)> {
    let line_groups = Rc::new(RefCell::new(Vec::new()));
    diffs.foreach(
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
    )?;

    let header_groups = Rc::new(RefCell::new(Vec::new()));
    diffs.foreach(
        &mut |_delta, _num| {
            header_groups.borrow_mut().push(Vec::new());
            true
        },
        None,
        Some(
            &mut |_delta, _hunk| match std::str::from_utf8(_hunk.header()) {
                Ok(c) => {
                    let mut content = c.to_string();
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
                }
                Err(_) => false,
            },
        ),
        None,
    )?;

    let mut result = Vec::new();
    diffs.foreach(
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
    )?;

    Ok((result, Stats::new(diffs.stats()?)))
}
