use crate::git::diff::{DiffEdit, DiffLine, EditType};
use anyhow::{anyhow, Context, Result};

// Heavily inspired by:
// https://blog.jcoglan.com/2017/02/12/the-myers-diff-algorithm-part-1/

pub struct Myers {
    a: Vec<DiffLine>,
    b: Vec<DiffLine>,
}

impl Myers {
    pub fn new(a: Vec<DiffLine>, b: Vec<DiffLine>) -> Myers {
        Myers { a, b }
    }

    pub fn diff(&self) -> Result<Vec<DiffEdit>> {
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

        let mut v = vec![0_isize; 2 * max as usize + 1];
        let mut trace = Vec::new();

        for d in 0..=max {
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
                    && get(v, k - 1).context("backtrack")? < get(v, k + 1).context("backtrack")?)
            {
                k + 1
            } else {
                k - 1
            };

            let prev_x = get(v, prev_k).context("backtrack")?;
            let prev_y = prev_x - prev_k;

            while x > prev_x && y > prev_y {
                res.push((x - 1, y - 1, x, y));
                x -= 1;
                y -= 1;
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

fn get<T: Clone>(vec: &[T], index: isize) -> Result<T> {
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

fn set<T>(vec: &mut [T], index: isize, value: T) {
    let len = vec.len() as isize;
    let actual_index = if index < 0 { len + index } else { index };

    if actual_index >= 0 && actual_index < len {
        vec[actual_index as usize] = value;
    } else {
        panic!("invalid vector access");
    }
}
