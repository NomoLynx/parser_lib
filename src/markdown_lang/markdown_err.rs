#[derive(Debug, Clone, PartialEq, Hash)]
pub enum MarkdownError {
    TableColumnNumberErr(u32, u32),   //first number is expected value, the 2nd value is actual value
}