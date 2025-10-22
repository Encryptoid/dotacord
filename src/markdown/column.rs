pub trait Column {
    fn len(&self) -> usize;
    fn calculate_width(&self) -> usize;
    fn format_header(&self, width: usize) -> String;
    fn format_cell(&self, row_index: usize, width: usize) -> String;
}

impl std::fmt::Debug for dyn Column {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Column")
    }
}

