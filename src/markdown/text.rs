use crate::fmt;

pub struct Text {
    header: String,
    values: Vec<String>,
    inline_code: bool,
}

impl Text {
    pub fn new(header: &str, values: Vec<String>) -> Self {
        Self {
            header: header.into(),
            values,
            inline_code: true,
        }
    }

    pub fn raw_text(mut self) -> Self {
        self.inline_code = false;
        self
    }

    fn format_with_style(&self, content: &str, width: usize) -> String {
        if self.inline_code {
            fmt!("`{:<width$}`", content, width = width)
        } else {
            fmt!("{:<width$}", content, width = width)
        }
    }
}

impl crate::markdown::Column for Text {
    fn len(&self) -> usize {
        self.values.len()
    }

    fn calculate_width(&self) -> usize {
        // NOTE: Discord renders inline code `text` as just the visible text,
        // so the rendered width is only text.len(), not including the backticks.
        let max_value_width = self
            .values
            .iter()
            .map(|value| value.len())
            .max()
            .unwrap_or(0);

        max_value_width.max(self.header.len())
    }

    fn format_header(&self, width: usize) -> String {
        self.format_with_style(&self.header, width)
    }

    fn format_cell(&self, row_index: usize, width: usize) -> String {
        self.format_with_style(&self.values[row_index], width)
    }
}

impl From<Text> for Box<dyn crate::markdown::Column> {
    fn from(t: Text) -> Self {
        Box::new(t)
    }
}
