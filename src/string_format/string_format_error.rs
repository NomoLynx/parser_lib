use std::fmt::{Display, Formatter};

use core_utils::core_utils_error::CoreUtilsError;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StringFormatError {
    ParsingConversionError,
    ConversionError(String),
    WrongType(String),
    TimeError,
}

impl Display for StringFormatError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParsingConversionError => write!(f, "parsing conversion error"),
            Self::ConversionError(msg) => write!(f, "conversion error: {msg}"),
            Self::WrongType(msg) => write!(f, "wrong type: {msg}"),
            Self::TimeError => write!(f, "time error"),
        }
    }
}

impl std::error::Error for StringFormatError {}

impl From<CoreUtilsError> for StringFormatError {
    fn from(value: CoreUtilsError) -> Self {
        match value {
            CoreUtilsError::ConversionError(msg) => Self::ConversionError(msg),
            CoreUtilsError::CannotRetrieveValue => {
                Self::ConversionError("cannot retrieve value".to_string())
            }
            CoreUtilsError::IncompatibleType => Self::WrongType("incompatible type".to_string()),
            CoreUtilsError::InvalidOperation => Self::WrongType("invalid operation".to_string()),
            CoreUtilsError::Other(msg) => Self::ConversionError(msg),
        }
    }
}
