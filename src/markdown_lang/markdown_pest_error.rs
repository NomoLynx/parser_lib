#[derive(Debug, Clone, PartialEq, Hash)]
pub enum MarkdownPestError {
    NoMatchWholeInput,
    PasringError,
    InvalidTableSize,
    InvalidLocation,
    InvalidFileFormat,
    MissingTopLevelHeader,
}