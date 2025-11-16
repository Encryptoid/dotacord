pub const LINK_SYMBOL: &str = "@";

#[derive(Debug, Clone)]
pub struct Section {
    pub title: String,
    pub lines: Vec<String>,
}

impl Section {
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

pub fn mask_link(url: &str, symbol: &str) -> String {
    fmt!("[{}]({})", symbol, url)
}

// Re-exports are provided via the `markdown` module under `crate::util::markdown`.