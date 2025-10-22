pub struct Link {
    text: String,
    urls: Vec<String>,
}

pub const LINK_SYMBOL: &str = "@";

pub fn mask_link(url: &str, symbol: &str) -> String {
    format!("[{}]({})", symbol, url)
}

impl Link {
    pub fn new(urls: Vec<String>) -> Self {
        Self {
            text: LINK_SYMBOL.to_owned(),
            urls,
        }
    }

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }
}

impl crate::markdown::Column for Link {
    fn len(&self) -> usize {
        self.urls.len()
    }

    fn calculate_width(&self) -> usize {
        // NOTE: Discord renders markdown links [text](url) as just the visible text,
        // so the rendered width is only text.len(), not the full markdown syntax length.
        self.text.len()
    }

    fn format_header(&self, width: usize) -> String {
        format!("{:<width$}", self.text, width = width)
    }

    fn format_cell(&self, row_index: usize, width: usize) -> String {
        // Just use the text for padding - the [](url) syntax isn't rendered by Discord
        format!("{:<width$}", mask_link(&self.urls[row_index], &self.text), width = width)
    }
}

impl From<Link> for Box<dyn crate::markdown::Column> {
    fn from(l: Link) -> Self {
        Box::new(l)
    }
}