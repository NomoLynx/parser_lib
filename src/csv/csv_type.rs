use chrono::{DateTime, NaiveDateTime};

use crate::common::get_suitable_type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum InferredType {
    Bool,
    Int,
    UInt,
    Float,
    DateTime,
    String,
}

impl InferredType {
    pub fn to_u8(self) -> u8 {
        match self {
            InferredType::Bool => 1,
            InferredType::Int => 2,
            InferredType::UInt => 3,
            InferredType::Float => 4,
            InferredType::DateTime => 5,
            InferredType::String => 6,
        }
    }

    pub fn from_u8(value: u8) -> Option<InferredType> {
        match value {
            1 => Some(InferredType::Bool),
            2 => Some(InferredType::Int),
            3 => Some(InferredType::UInt),
            4 => Some(InferredType::Float),
            5 => Some(InferredType::DateTime),
            6 => Some(InferredType::String),
            _ => None,
        }
    }
}

fn is_hex_number(s: &str) -> bool {
    if let Some(hex) = s.strip_prefix("0x")
        .or_else(|| s.strip_prefix("0X"))
    {
        i64::from_str_radix(hex.replace('_', "").as_str(), 16).is_ok()
    } else {
        false
    }
}

fn is_binary_number(s: &str) -> bool {
    if let Some(bin) = s.strip_prefix("0b")
        .or_else(|| s.strip_prefix("0B"))
    {
        let cleaned = bin.replace('_', "");
        !cleaned.is_empty() &&
        cleaned.chars().all(|c| c == '0' || c == '1')
    } else {
        false
    }
}

fn is_octal_number(s: &str) -> bool {
    if let Some(oct) = s.strip_prefix("0o")
        .or_else(|| s.strip_prefix("0O"))
    {
        let cleaned = oct.replace('_', "");
        !cleaned.is_empty()
            && cleaned.chars().all(|c| matches!(c, '0'..='7'))
    } else {
        false
    }
}

pub (crate) fn infer_cell(s: &str) -> Option<InferredType> {
    let s = s.trim();
    if s.is_empty() {
        return None; // signals optional, no type impact
    }

    if matches!(s.to_lowercase().as_str(), "true" | "false" | "0" | "1") {
        Some(InferredType::Bool)
    } else if s.parse::<i64>().is_ok() || 
            is_hex_number(s) || is_octal_number(s) ||
            is_binary_number(s) {
                if get_suitable_type(s).unwrap() {  //TODO: need to handle error here
                    Some(InferredType::Int)
                } else {
                    Some(InferredType::UInt)
                }
            } 
    else if s.parse::<f64>().is_ok() {
        Some(InferredType::Float)
    } else if DateTime::parse_from_rfc3339(s).is_ok()
        || NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").is_ok()
    {
        Some(InferredType::DateTime)
    } else {
        Some(InferredType::String)
    }
}

pub fn infer_column<'a, I>(cells: I) -> (InferredType, bool)
where
    I: IntoIterator<Item = &'a str>,
{
    let mut ty = InferredType::Bool;
    let mut optional = false;

    let mut iter = cells.into_iter();
    iter.next(); // skip the first element (column name)

    for cell in iter {
        match infer_cell(cell) {
            Some(t) => ty = ty.max(t),
            None => optional = true,
        }
    }

    (ty, optional)
}

pub fn rust_type(t: InferredType, optional: bool) -> String {
    let base = match t {
        InferredType::Bool => "bool",
        InferredType::Int => "i64",
        InferredType::UInt => "u64",
        InferredType::Float => "f64",
        InferredType::DateTime => "chrono::DateTime<chrono::Utc>",
        InferredType::String => "String",
    };

    if optional {
        format!("Option<{}>", base)
    } else {
        base.to_string()
    }
}

/// Infers the Rust type for a CSV column given its cell values.
pub fn infer_rust_type_from_column<'a, I>(cells: I) -> String
where
    I: IntoIterator<Item = &'a str>,
{
    let (inferred_type, optional) = infer_column(cells);
    rust_type(inferred_type, optional)
}