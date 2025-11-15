pub struct LeaderboardSection {
    pub title: String,
    pub lines: Vec<String>,
}

impl LeaderboardSection {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            lines: Vec::new(),
        }
    }

    pub fn add_line(&mut self, line: impl Into<String>) {
        self.lines.push(line.into());
    }
}
