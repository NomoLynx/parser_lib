use crate::markdown_lang::{markdown_err::MarkdownError, markdown_pest_error::MarkdownPestError};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ErrorSourceFileLocation(pub String, pub u32);

impl std::fmt::Display for ErrorSourceFileLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:0", self.0, self.1)
    }
}

impl From<(String, u32)> for ErrorSourceFileLocation {
    fn from(src: (String, u32)) -> Self {
        ErrorSourceFileLocation(src.0, src.1)
    }
}

impl std::fmt::Debug for ErrorSourceFileLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:0", self.0, self.1)
    }
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum EscapeStringError {
    EscapeAtEndOfString,
    UnrecognizedEscapedChar(char),
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum ParsingError {
    IOError,
    ParsingConversionError,
    EscapeStringError(EscapeStringError),
    MarkdownPestError(MarkdownPestError),
    MarkdownErr(MarkdownError),
    NoFound(ErrorSourceFileLocation, String),
}

impl std::fmt::Display for EscapeStringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EscapeAtEndOfString => write!(f, "Escape character at the end of the string"),
            Self::UnrecognizedEscapedChar(c) => write!(f, "Unrecognized escaped char: '{}'", c),
        }
    }
}

impl std::error::Error for EscapeStringError {}