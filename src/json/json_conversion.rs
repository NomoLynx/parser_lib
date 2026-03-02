use chrono::{DateTime, Utc};

use crate::common::*;
use crate::json::*;

pub trait AsI64 {
    fn as_i64(&self) -> Option<i64>;
    fn as_f64(&self) -> Option<f64>;
    fn as_u64(&self) -> Option<u64>;
}

impl AsI64 for str {
    fn as_i64(&self) -> Option<i64> {
        self.trim().parse::<i64>().ok()
    }

    fn as_u64(&self) -> Option<u64> {
        self.trim().parse::<u64>().ok()
    }

    fn as_f64(&self) -> Option<f64> {
        self.trim().parse().ok()
    }
}

pub fn json_to_bool(v: &Value) -> Option<bool> {
    match v {
        Value::Boolean(b) => Some(*b),
        Value::Number(n) => {
            if n.as_i64() == Some(1) {
                Some(true)
            } else if n.as_i64() == Some(0) {
                Some(false)
            } else {
                None
            }
        }
        Value::String(s) => match s.trim().to_ascii_lowercase().as_str() {
            "true" | "1" | "yes" | "y" => Some(true),
            "false" | "0" | "no" | "n" => Some(false),
            _ => None,
        },
        _ => None,
    }
}

pub fn json_to_bool_value(v: &Value) -> bool {
    match json_to_bool(v) {
        Some(b) => b,
        None => false,
    }
}

pub fn json_to_int(v: &Value) -> Option<i64> {
    match v {
        Value::Number(n) => n.as_i64(),
        Value::String(s) => s.trim().parse::<i64>().ok(),
        _ => None,
    }
}

pub fn json_to_int_value(v: &Value) -> i64 {
    json_to_int(v).expect("expect an int value from json")
}

pub fn json_to_uint(v: &Value) -> Option<u64> {
    match v {
        Value::Number(n) => n.as_u64(),
        Value::String(s) => s.trim().parse::<u64>().ok(),
        _ => None,
    }
}

pub fn json_to_uint_value(v: &Value) -> u64 {
    json_to_uint(v).expect("expect a uint value from json")
}

pub fn json_to_float(v: &Value) -> Option<f64> {
    match v {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.trim().parse::<f64>().ok(),
        _ => None,
    }
}

pub fn json_to_float_value(v: &Value) -> f64 {
    json_to_float(v).expect("expect a float value from json")
}

pub fn json_to_datetime(v: &Value) -> Option<DateTime<Utc>> {
    match v {
        Value::String(s) => {
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
        _ => None,
    }
}

pub fn json_to_datetime_value(v: &Value) -> DateTime<Utc> {
    json_to_datetime(v).expect("expect a datetime value from json")
}

pub fn json_to_string(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(strip_quotes(s).to_string()),
        // optional: lossy stringify for numbers/bools
        Value::Number(n) => Some(n.to_string()),
        Value::Boolean(b) => Some(b.to_string()),
        _ => None,
    }
}

pub fn json_to_string_value(v: &Value) -> String {
    match json_to_string(v) {
        Some(s) => s,
        None => {
            panic!("expect a string value from json but got: {:?}", v);
        }
    }
}

pub fn json_to_array<T>(
    v: &Value,
    elem: fn(&Value) -> T,
) -> Option<Vec<T>> {
    match v {
        Value::Array(arr) => {
            let r = arr.iter()
                .map(|x| elem(x))
                .collect::<Vec<_>>();
            Some(r)
        }
        _ => None,
    }
}

