use chrono::NaiveDateTime;
use core::fmt;
use git2::{DiffStats, Error, Repository, Sort};
use std::{cell::RefCell, path::PathBuf, rc::Rc};

#[derive(Debug, Clone)]
pub struct Diff {
    old_file: PathBuf,
    new_file: PathBuf,
    pub headers: Vec<Header>,
    pub lines: Vec<Line>,
    pub content: String,
    pub origins_content: String,
    pub lines_content: String,
    pub header_indices: Vec<usize>,
    pub insertion_indices: Vec<usize>,
    pub deletion_indices: Vec<usize>,
    pub neutral_indices: Vec<usize>,
}

impl Diff {
    fn new(old_file: PathBuf, new_file: PathBuf, headers: Vec<Header>, lines: Vec<Line>) -> Diff {
        let longest_line = get_longest_line(&lines);

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

            while line_no.len() != longest_line {
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
            lines_content,
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

fn get_longest_line(lines: &Vec<Line>) -> usize {
    let mut longest_line = 0;
    for line in lines {
        let line_no = match line.origin {
            '+' => line.new_lineno.unwrap_or(0),
            '-' => line.old_lineno.unwrap_or(0),
            _ => line.new_lineno.unwrap_or(0),
        };

        if line_no > longest_line {
            longest_line = line_no;
        }
    }

    longest_line.to_string().len()
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
    pub content: String,
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
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.origin, self.content)
    }
}

#[derive(Debug, Clone)]
pub struct Stats {
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
}

impl Stats {
    fn new(diff_stats: DiffStats) -> Stats {
        Stats {
            files_changed: diff_stats.files_changed(),
            insertions: diff_stats.insertions(),
            deletions: diff_stats.deletions(),
        }
    }
}

#[derive(Debug)]
pub struct DiffParsingError;

pub fn get_staged_diffs(path: &String) -> Result<(Vec<Diff>, Stats), DiffParsingError> {
    let repo = Repository::open(path).map_err(|_| DiffParsingError)?;
    let head = repo
        .head()
        .map_err(|_| DiffParsingError)?
        .peel_to_tree()
        .map_err(|_| DiffParsingError)?;
    let diffs = repo
        .diff_tree_to_index(Some(&head), None, None)
        .map_err(|_| DiffParsingError)?;

    parse_diffs(diffs)
}

pub fn get_diffs(path: &String) -> Result<(Vec<Diff>, Stats), DiffParsingError> {
    let repo = Repository::open(path).map_err(|_| DiffParsingError)?;
    let diffs = repo
        .diff_index_to_workdir(None, None)
        .map_err(|_| DiffParsingError)?;

    parse_diffs(diffs)
}

fn parse_diffs(diffs: git2::Diff) -> Result<(Vec<Diff>, Stats), DiffParsingError> {
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

    Ok((
        result,
        Stats::new(diffs.stats().map_err(|_| DiffParsingError)?),
    ))
}

#[derive(Debug, Clone)]
pub struct Commit {
    pub id: String,
    pub author: Author,
    pub message: String,
    pub time: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct Author {
    pub name: String,
    pub email: String,
}

pub fn get_log(path: &String) -> Result<Vec<Commit>, Error> {
    let repo = Repository::open(path)?;
    let mut revwalk = repo.revwalk()?;
    revwalk.set_sorting(Sort::TIME)?;
    revwalk.push_head()?;

    let mut commits = Vec::new();

    for id in revwalk {
        let id = id?;
        let commit = repo.find_commit(id)?;

        let author = Author {
            name: commit.author().name().unwrap_or("").to_owned(),
            email: commit.author().email().unwrap_or("").to_owned(),
        };

        let commit = Commit {
            id: id.to_string(),
            author,
            message: commit.message().unwrap_or("").to_owned(),
            time: NaiveDateTime::from_timestamp_opt(
                commit.time().seconds() + commit.time().offset_minutes() as i64 * 60,
                0,
            )
            .unwrap_or_default(),
        };

        commits.push(commit);
    }

    Ok(commits)
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
