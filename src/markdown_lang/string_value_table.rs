pub type StringValueTableRow = Vec<String>;

#[derive(Debug, Clone)]
pub struct StringValueTable (Vec<StringValueTableRow>);

impl StringValueTable {
    pub fn get_table(&self) -> &Vec<StringValueTableRow> {
        &self.0
    }

    pub fn new(table:Vec<StringValueTableRow>) -> Self {
        Self(table)
    }
}

impl Default for StringValueTable {
    fn default() -> Self {
        Self::new(Vec::default())
    }
}