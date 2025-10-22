use crate::markdown::Column;
use crate::leaderboard::section::Section;

pub struct TableBuilder {
    title: String,
    columns: Vec<Box<dyn Column>>,
    row_count: Option<usize>,
}

impl TableBuilder {
    pub(crate) fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            columns: Vec::new(),
            row_count: None,
        }
    }

    pub(crate) fn add_column(mut self, column: impl Into<Box<dyn Column>>) -> Self {
        let column = column.into();
        let column_len = column.len();

        match self.row_count {
            None => self.row_count = Some(column_len),
            Some(existing) if existing == column_len => {}
            Some(existing) => {
                panic!(
                    "column length mismatch: expected {} rows but received {}",
                    existing, column_len
                );
            }
        }

        self.columns.push(column);
        self
    }

    pub(crate) fn build(self) -> Section {
        let mut section = Section::new(&self.title);

        let row_count = self.row_count.unwrap_or(0);
        if row_count == 0 {
            section.add_line("No data available.".to_string());
            return section;
        }

        self.build_table_content(&mut section, row_count);
        section
    }

    fn build_table_content(&self, section: &mut Section, row_count: usize) {
        // Calculate column widths
        let widths: Vec<usize> = self.columns.iter().map(|col| col.calculate_width()).collect();

        // Build header row (text cells wrapped in inline code; link cells just padded)
        let header_cells: Vec<String> = self
            .columns
            .iter()
            .zip(&widths)
            .map(|(col, &width)| col.format_header(width))
            .collect();
        let header_row = format!("| {} |", header_cells.join(" | "));
        section.add_line(header_row);

        // Build data rows
        for row_index in 0..row_count {
            let cells: Vec<String> = self
                .columns
                .iter()
                .zip(&widths)
                .map(|(col, &width)| col.format_cell(row_index, width))
                .collect();
            let row_str = format!("| {} |", cells.join(" | "));
            section.add_line(row_str);
        }
    }
}