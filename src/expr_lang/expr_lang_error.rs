use core_utils::core_utils_error::CoreUtilsError;

#[derive(Debug)]
pub enum ExprLangError {
    CannotRetrieveValue,
    InvalidTypeConversion,
    InvalidExpression,
    ParameterError,
    IncompatibleType,
    InvalidOperation,
    Other(String),
    NoFound((String, u32), String),
}

impl From<CoreUtilsError> for ExprLangError {
    fn from(e: CoreUtilsError) -> Self {
        match e {
            CoreUtilsError::ConversionError(_) => ExprLangError::ParameterError,
            CoreUtilsError::CannotRetrieveValue => ExprLangError::CannotRetrieveValue,
            CoreUtilsError::IncompatibleType => ExprLangError::IncompatibleType,
            CoreUtilsError::InvalidOperation => ExprLangError::InvalidOperation,
            CoreUtilsError::Other(msg) => ExprLangError::Other(msg),
        }
    }
}