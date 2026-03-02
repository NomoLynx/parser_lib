pub mod json_pest;
pub mod json_parse_error;
pub mod json_type;
pub mod json_conversion;

pub use json_pest::*;
pub use json_parse_error::*;
pub use json_type::*;
pub use json_conversion::*;

use pest::Parser;
use crate::common::debug::*;

pub fn parse_json(input: &str) -> Result<Object, JsonParseError> {
    let mut parsed = JSONParser::parse(Rule::json, input)
        .map_err(|e| JsonParseError::GeneralError(format!("'{}' has error {}", e.line(), e.to_string())))?;
    let object_pair = parsed.next().unwrap();
    Object::from_pair(object_pair)
}

pub fn parse_json_from_file(file_path: &str) -> Result<Object, JsonParseError> {
    let input = std::fs::read_to_string(file_path)
        .map_err(|e| JsonParseError::GeneralError(format!("Failed to read file '{}': {}", file_path, e)))?;
    
    match parse_json(&input) {
        Ok(obj) => Ok(obj),
        Err(e) => {
            error_string(format!("Error parsing JSON from file '{}': {:?}", file_path, e));
            Err(e)
        }
    }
}