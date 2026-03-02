use chrono::{DateTime, Utc};

use crate::csv::InferredType;

pub fn str_to_bool(s: &str) -> Option<bool> {
    match s.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "y" => Some(true),
        "false" | "0" | "no" | "n" => Some(false),
        _ => None,
    }
}

pub fn str_to_int(s: &str) -> Option<i64> {
    s.trim().parse::<i64>()
        .or_else(|_| i64::from_str_radix(s.trim(), 16))
        .or_else(|_| i64::from_str_radix(s.trim(), 2))
        .ok()
}

pub fn str_to_uint(s: &str) -> Option<u64> {
    s.trim().parse::<u64>()
        .or_else(|_| u64::from_str_radix(s.trim(), 16))
        .or_else(|_| u64::from_str_radix(s.trim(), 2))
        .ok()
}

pub fn str_to_float(s: &str) -> Option<f64> {
    s.trim().parse::<f64>().ok()
}

pub fn str_to_datetime(s: &str) -> Option<DateTime<Utc>> {
    let s = s.trim();
    // Try to parse with timezone info
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        Some(dt.with_timezone(&Utc))
    } else if let Ok(dt) = DateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S %z") {
        Some(dt.with_timezone(&Utc))
    } else if let Ok(dt) = DateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%z") {
        Some(dt.with_timezone(&Utc))
    }
    // Try to parse without timezone info, assume UTC
    else if let Ok(naive_dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        Some(DateTime::<Utc>::from_naive_utc_and_offset(naive_dt, Utc))
    } else if let Ok(naive_dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
        Some(DateTime::<Utc>::from_naive_utc_and_offset(naive_dt, Utc))
    } else {
        None
    }
}

pub fn str_to_string(s: &str) -> Option<String> {
    Some(s.to_owned())
}

pub fn convert_function(inferred_type_id:&InferredType) -> Option<String> {
    match inferred_type_id {
        InferredType::Bool => Some("str_to_bool".to_string()),
        InferredType::Int => Some("str_to_int".to_string()),
        InferredType::UInt => Some("str_to_uint".to_string()),
        InferredType::Float => Some("str_to_float".to_string()),
        InferredType::DateTime => Some("str_to_datetime".to_string()),
        InferredType::String => Some("str_to_string".to_string()),
    }
}