pub(crate) trait HasPlayerId {
    fn player_id(&self) -> i64;
}

const LINK_SYMBOL: &str = "@";

#[derive(Debug, Clone)]
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

pub(crate) struct Column<'a, R: HasPlayerId> {
    header: &'a str,
    value_fn: Box<dyn Fn(&R) -> String + 'a>,
    width_fn: Option<Box<dyn Fn(&R) -> usize + 'a>>,
    match_id_fn: Option<Box<dyn Fn(&R) -> Option<i64> + 'a>>,
}

impl<'a, R: HasPlayerId> Column<'a, R> {
    pub(crate) fn new(header: &'a str, value_fn: impl Fn(&R) -> String + 'a) -> Self {
        Self {
            header,
            value_fn: Box::new(value_fn),
            width_fn: None,
            match_id_fn: None,
        }
    }

    #[allow(dead_code)]
    fn with_width_fn(mut self, width_fn: impl Fn(&R) -> usize + 'a) -> Self {
        self.width_fn = Some(Box::new(width_fn));
        self
    }

    pub(crate) fn with_match_id_fn(mut self, match_id_fn: impl Fn(&R) -> Option<i64> + 'a) -> Self {
        self.match_id_fn = Some(Box::new(match_id_fn));
        self
    }

    pub(crate) fn calculate_width(&self, stats: &Vec<&R>) -> usize {
        let content_width = stats
            .iter()
            .map(|s| {
                if let Some(ref width_fn) = self.width_fn {
                    width_fn(s)
                } else {
                    (self.value_fn)(s).len()
                }
            })
            .max()
            .unwrap_or(0);
        content_width.max(self.header.len())
    }

    fn format_cell(&self, value: &str, width: usize) -> String {
        format!("{:<width$}", value, width = width)
    }
}

pub(crate) struct TableBuilder<'a, R: HasPlayerId> {
    title: String,
    columns: Vec<Column<'a, R>>,
    link_fn: Option<Box<dyn Fn(&R) -> String + 'a>>,
}

impl<'a, R: HasPlayerId> TableBuilder<'a, R> {
    pub(crate) fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            columns: Vec::new(),
            link_fn: None,
        }
    }

    pub(crate) fn add_column(mut self, column: Column<'a, R>) -> Self {
        self.columns.push(column);
        self
    }

    /// Optionally set a custom link generator for each row. If not set, defaults to player profile links.
    pub(crate) fn with_link_fn(mut self, link_fn: impl Fn(&R) -> String + 'a) -> Self {
        self.link_fn = Some(Box::new(link_fn));
        self
    }

    pub(crate) fn build(&self, sorted_stats: Vec<&R>) -> LeaderboardSection {
        let mut section = LeaderboardSection::new(&self.title);

        if sorted_stats.is_empty() {
            section.add_line("No data available.".to_string());
            return section;
        }

        self.build_table_content(&mut section, sorted_stats);
        section
    }

    fn build_table_content(&self, section: &mut LeaderboardSection, filtered: Vec<&R>) {
        // Calculate column widths
        let widths: Vec<usize> = self
            .columns
            .iter()
            .map(|col| col.calculate_width(&filtered))
            .collect();

        // Build header row (each cell is wrapped in inline code; pipes remain outside)
        let header_cells: Vec<String> = self
            .columns
            .iter()
            .zip(&widths)
            .map(|(col, &width)| col.format_cell(col.header, width))
            .collect();
        let header_cells_wrapped: Vec<String> =
            header_cells.iter().map(|c| format!("`{}`", c)).collect();
        let header_row = format!("| {} |", header_cells_wrapped.join(" | "));
        section.add_line(format!("{} {}", LINK_SYMBOL, header_row));

        // Build data rows
        for row in filtered {
            let cells: Vec<String> = self
                .columns
                .iter()
                .zip(&widths)
                .map(|(col, &width)| {
                    let value = (col.value_fn)(row);
                    col.format_cell(&value, width)
                })
                .collect();
            let wrapped_cells: Vec<String> = cells.iter().map(|c| format!("`{}`", c)).collect();
            let row_str = format!("| {} |", wrapped_cells.join(" | "));
            // Prefer a match link if any column provides a match id for this row.
            let mut match_link: Option<String> = None;
            for col in &self.columns {
                if let Some(ref mf) = col.match_id_fn {
                    if let Some(mid) = mf(row) {
                        if mid > 0 {
                            match_link = Some(fmt_match_url(LINK_SYMBOL, mid));
                            break;
                        }
                    }
                }
            }
            let link = if let Some(ml) = match_link {
                ml
            } else if let Some(ref lf) = self.link_fn {
                lf(row)
            } else {
                fmt_profile_url(LINK_SYMBOL, row.player_id())
            };
            let line = format!("{} {}", link, row_str);
            section.add_line(line);
        }
    }
}

pub fn fmt_profile_url(text: &str, player_id: i64) -> String {
    format!(
        "[{}](<https://www.opendota.com/players/{}>)",
        text, player_id
    )
}

/// Format a link to a match page for the given match id.
pub fn fmt_match_url(text: &str, match_id: i64) -> String {
    format!("[{}](https://www.opendota.com/matches/{})", text, match_id)
}
