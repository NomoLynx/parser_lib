use std::collections::BTreeMap;

use crate::{common::*, csv::*, json::Value};
use rust_macro::*;

#[derive(Debug, Clone, GenIsEnumVariant)]
pub enum JsonType {
    Scalar(InferredType),
    Array(Box<JsonType>),
    Object(String, BTreeMap<String, JsonType>),
    Optional(Box<JsonType>),
}

pub fn is_i64(s: &str) -> bool {
    let s = s.trim();

    if s.is_empty() {
        return false;
    }

    let bytes = s.as_bytes();

    // Optional leading '-'
    let start = if bytes[0] == b'-' { 1 } else { 0 };

    if start == bytes.len() {
        return false;
    }

    // Digits only
    if !bytes[start..].iter().all(|b| b.is_ascii_digit()) {
        return false;
    }

    // Range check
    s.parse::<i64>().is_ok()
}

fn infer_scalar_from_str(s: &str) -> InferredType {
    match infer_cell(s) {
        Some(ty) => ty,
        None => InferredType::String,
    }
}

fn merge_json_type(a: JsonType, b: JsonType) -> JsonType {
    use JsonType::*;

    match (a, b) {
        (Scalar(x), Scalar(y)) => Scalar(x.max(y)),
        (Array(x), Array(y)) => Array(Box::new(merge_json_type(*x, *y))),
        (Optional(x), y) | (y, Optional(x)) => Optional(Box::new(merge_json_type(*x, y))),
        (Object(m, fields0), Object(_, fields1)) => {
            let mut merged_fields = BTreeMap::new();
            for (k, v0) in fields0 {
                if let Some(v1) = fields1.get(&k) {
                    merged_fields.insert(k.clone(), merge_json_type(v0, v1.clone()));
                } else {
                    merged_fields.insert(k.clone(), JsonType::Optional(Box::new(v0)));
                }
            }
            for (k, v1) in fields1 {
                if !merged_fields.contains_key(&k) {
                    merged_fields.insert(k.clone(), JsonType::Optional(Box::new(v1)));
                }
            }
            Object(m, merged_fields)
        }
        _ => Scalar(InferredType::String), // conflict fallback
    }
}

pub fn infer_json(name:String, v: &Value) -> JsonType {
    match v {
        Value::Null => JsonType::Optional(Box::new(JsonType::Scalar(InferredType::String))),

        Value::Boolean(_) => JsonType::Scalar(InferredType::Bool),

        Value::Number(n) => { 
            match get_suitable_type(n) {
                Ok(true) => {
                    JsonType::Scalar(InferredType::Int)
                }
                Ok(false) => {
                    JsonType::Scalar(InferredType::UInt)
                }
                Err(_) => {
                    JsonType::Scalar(InferredType::Float)
                }
            }
        }

        Value::String(s) => JsonType::Scalar(infer_scalar_from_str(s)),

        Value::Array(arr) => {
            if arr.is_empty() {
                return JsonType::Array(Box::new(JsonType::Scalar(InferredType::String)));
            }

            if arr.iter().any(|x| x.is_null()) {
                let inner = arr
                    .iter()
                    .filter(|x| !x.is_null())
                    .map(|x| infer_json(format!("ArrayItem_{name}"), x))
                    .reduce(merge_json_type)
                    .unwrap_or(JsonType::Scalar(InferredType::String));
                if inner.is_optional() {
                    JsonType::Array(Box::new(inner))
                } else {
                    JsonType::Array(Box::new(JsonType::Optional(Box::new(inner))))
                }
            }
            else {
                let inner = arr
                    .iter()
                    .map(|x| infer_json(format!("ArrayItem_{name}"), x))
                    .reduce(merge_json_type)
                    .unwrap_or(JsonType::Scalar(InferredType::String));
                JsonType::Array(Box::new(inner))
            }
        }

        Value::Object(map) => {
            let mut fields = BTreeMap::new();
            for p in map.get_pairs() {
                let (k ,v) = p.into();
                let type_name = if name.is_empty() { format!("{k}") } else { format!("{name}_{k}") };
                fields.insert(k.clone(), infer_json(pascal(&type_name), v));
            }
            JsonType::Object(name.clone(), fields)
        }
    }
}

pub fn rust_scalar(t: InferredType) -> &'static str {
    match t {
        InferredType::Bool => "bool",
        InferredType::Int => "i64",
        InferredType::UInt => "u64",
        InferredType::Float => "f64",
        InferredType::DateTime => "chrono::DateTime<chrono::Utc>",
        InferredType::String => "String",
    }
}

pub fn emit_type(t: &JsonType) -> String {
    match t {
        JsonType::Scalar(s) => rust_scalar(*s).into(),
        JsonType::Array(inner) => format!("Vec<{}>", emit_type(inner)),
        JsonType::Optional(inner) => format!("Option<{}>", emit_type(inner)),
        JsonType::Object(name, _fields) => name.into(),
    }
}
