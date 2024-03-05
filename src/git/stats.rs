use git2::DiffStats;

#[derive(Debug, Clone)]
pub struct Stats {
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
}

impl Stats {
    pub fn new(diff_stats: DiffStats) -> Stats {
        Stats {
            files_changed: diff_stats.files_changed(),
            insertions: diff_stats.insertions(),
            deletions: diff_stats.deletions(),
        }
    }
}
