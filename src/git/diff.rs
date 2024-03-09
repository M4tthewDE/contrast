use std::fmt::Display;

use anyhow::{anyhow, Context, Result};

// Heavily inspired by:
// https://blog.jcoglan.com/2017/02/12/the-myers-diff-algorithm-part-1/

#[derive(Debug, Clone)]
struct DiffLine {
    number: usize,
    text: String,
}

impl Display for DiffLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} {}", self.number, self.text)
    }
}

fn calculate_diff(a: &str, b: &str) -> Result<Vec<DiffEdit>> {
    let a_lines = get_lines(a);
    let b_lines = get_lines(b);

    Myers::new(a_lines, b_lines).diff()
}

fn get_lines(content: &str) -> Vec<DiffLine> {
    let mut lines = Vec::new();
    for (i, line) in content.lines().enumerate() {
        lines.push(DiffLine {
            number: i,
            text: line.to_string(),
        })
    }

    lines
}

enum EditType {
    Ins,
    Del,
    Eql,
}

struct DiffEdit {
    typ: EditType,
    a_line: Option<DiffLine>,
    b_line: Option<DiffLine>,
}

impl Display for DiffEdit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.typ {
            EditType::Ins => write!(f, "I {}", self.b_line.clone().unwrap()),
            EditType::Del => write!(f, "D {}", self.a_line.clone().unwrap()),
            EditType::Eql => write!(f, "  {}", self.a_line.clone().unwrap()),
        }
    }
}

impl DiffEdit {
    fn new(typ: EditType, a_line: Option<DiffLine>, b_line: Option<DiffLine>) -> DiffEdit {
        match typ {
            EditType::Ins => assert!(b_line.is_some()),
            EditType::Del => assert!(a_line.is_some()),
            EditType::Eql => assert!(a_line.is_some() && b_line.is_some()),
        }
        DiffEdit {
            typ,
            a_line,
            b_line,
        }
    }
}

struct Myers {
    a: Vec<DiffLine>,
    b: Vec<DiffLine>,
}

impl Myers {
    fn new(a: Vec<DiffLine>, b: Vec<DiffLine>) -> Myers {
        Myers { a, b }
    }

    fn diff(&self) -> Result<Vec<DiffEdit>> {
        let mut diff = Vec::new();

        for (prev_x, prev_y, x, y) in self.backtrack()? {
            let (a_line, b_line) = (get(&self.a, prev_x).ok(), get(&self.b, prev_y).ok());

            let edit = if x == prev_x {
                DiffEdit::new(EditType::Ins, None, b_line)
            } else if y == prev_y {
                DiffEdit::new(EditType::Del, a_line, None)
            } else {
                DiffEdit::new(EditType::Eql, a_line, b_line)
            };

            diff.insert(0, edit);
        }

        Ok(diff)
    }

    fn shortest_edit(&self) -> Result<Vec<Vec<isize>>> {
        let (n, m) = (self.a.len() as isize, self.b.len() as isize);
        let max = n + m;

        let mut v = vec![0 as isize; 2 * max as usize + 1];
        let mut trace = Vec::new();

        for d in 0..=max as isize {
            trace.push(v.clone());
            for k in (-d..=d).step_by(2) {
                let mut x = if k == -d
                    || (k != d
                        && get(&v, k - 1).context("shortest_edit")?
                            < get(&v, k + 1).context("shortest_edit")?)
                {
                    get(&v, k + 1).context("shortest_edit")?
                } else {
                    get(&v, k - 1).context("shortest_edit")? + 1
                };

                let mut y = x - k;

                while x < n
                    && y < m
                    && get(&self.a, x).context("shortest_edit")?.text
                        == get(&self.b, y).context("shortest_edit")?.text
                {
                    x += 1;
                    y += 1;
                }

                set(&mut v, k, x);

                if x >= n && y >= m {
                    return Ok(trace);
                }
            }
        }

        Err(anyhow!("no shortest edit found"))
    }

    fn backtrack(&self) -> Result<Vec<(isize, isize, isize, isize)>> {
        let (mut x, mut y) = (self.a.len() as isize, self.b.len() as isize);

        let mut res = Vec::new();
        for (d, v) in self.shortest_edit()?.iter().enumerate().rev() {
            let d = d as isize;
            let k = x - y;

            let prev_k = if k == -d
                || (k != d
                    && get(&v, k - 1).context("backtrack")?
                        < get(&v, k + 1).context("backtrack")?)
            {
                k + 1
            } else {
                k - 1
            };

            let prev_x = get(&v, prev_k).context("backtrack")?;
            let prev_y = prev_x - prev_k;

            while x > prev_x && y > prev_y {
                res.push((x - 1, y - 1, x, y));
                x = x - 1;
                y = y - 1;
            }

            if d > 0 {
                res.push((prev_x, prev_y, x, y));
            }

            x = prev_x;
            y = prev_y;
        }

        Ok(res)
    }
}

fn get<T: Clone>(vec: &Vec<T>, index: isize) -> Result<T> {
    if index >= 0 {
        vec.get(index as usize)
            .cloned()
            .context("invalid vector access")
    } else {
        vec.get((vec.len() as isize + index) as usize)
            .cloned()
            .context("invalid vector access")
    }
}

fn set<T>(vec: &mut Vec<T>, index: isize, value: T) {
    let len = vec.len() as isize;
    let actual_index = if index < 0 { len + index } else { index };

    if actual_index >= 0 && actual_index < len {
        vec[actual_index as usize] = value;
    } else {
        panic!("invalid vector access");
    }
}

#[cfg(test)]
mod tests {
    use super::calculate_diff;

    #[test]
    fn test_calculate_diff() {
        let old = include_str!("../../tests/data/a");
        let new = include_str!("../../tests/data/b");

        let diff = calculate_diff(old, new).unwrap();
        dbg!(&diff.len());
        for d in diff {
            dbg!(format!("{}", d));
        }
        todo!("implement");
    }
}
