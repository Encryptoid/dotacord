/// A simple builder for constructing markdown strings.
#[derive(Debug, Default, Clone)]
pub struct MarkdownBuilder {
    content: String,
}

impl MarkdownBuilder {
    pub fn new() -> Self {
        Self {
            content: String::new(),
        }
    }

    pub fn build(self) -> String {
        self.content
    }

    pub fn as_str(&self) -> &str {
        &self.content
    }

    pub fn nl(mut self) -> Self {
        self.content.push('\n');
        self
    }

    pub fn text(&mut self, text: &str) -> &mut Self {
        self.content.push_str(text);
        self
    }

    pub fn line(&mut self, text: &str) -> &mut Self {
        self.content.push_str(text);
        self.content.push('\n');
        self
    }

    pub fn list<T, F>(mut self, items: &[T], format_fn: F) -> Self
    where
        F: Fn(&mut Self, &T),
    {
        for item in items {
            self.content.push_str("- ");
            format_fn(&mut self, item);
            self.content.push('\n');
        }
        self
    }

    pub fn heading1(mut self, text: &str) -> Self {
        self.content.push_str("# ");
        self.content.push_str(text);
        self.content.push('\n');
        self
    }

    pub fn url(&mut self, display_text: &str, url: &str) -> &mut Self {
        self.content.push('[');
        self.content.push_str(display_text);
        self.content.push_str("](");
        self.content.push_str(url);
        self.content.push(')');
        self
    }

    pub fn list_item(mut self) -> Self {
        self.content.push_str("- ");
        self
    }
}
