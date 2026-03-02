use std::ops::Index;

use pest::iterators::Pair;
use pest_derive::Parser;
use rust_macro::{Accessors, EnumAccessors, GenIsEnumVariant};

use crate::{common::*, json::JsonParseError};

#[derive(Parser)]
#[grammar = "json/json.pest"] 
pub struct JSONParser;

/// Represents different JSON value types.
#[derive(Debug, GenIsEnumVariant, EnumAccessors)]
pub enum Value {
    Object(Object),
    Array(Vec<Value>),
    String(String),
    Number(String),
    Boolean(bool),
    Null,
}

impl Value {
    /// Constructs a `Value` from a pest `Pair<Rule>`.
    pub fn from_pair(pair: Pair<Rule>) -> Result<Self, JsonParseError> {
        match pair.as_rule() {
            Rule::object => Ok(Value::Object(Object::from_pair(pair)?)),
            Rule::array => Ok(Value::Array(
                pair.into_inner()
                    .map(|x| Value::from_pair(x).unwrap())
                    .collect()
            )),
            Rule::string => Ok(Value::String(strip_quotes(pair.as_str()).to_string())),
            Rule::number => Ok(Value::Number(pair.as_str().to_string())), // Store number as string to avoid precision issues.
            Rule::boolean => Ok(Value::Boolean(pair.as_str().to_lowercase() == "true")),
            Rule::null => Ok(Value::Null),
            _ => Err(JsonParseError::GeneralError(format!("Unexpected value type: {:?}", pair))),
        }
    }

    /// Converts `Value` into a JSON string representation.
    pub fn to_json(&self) -> String {
        match self {
            Value::Object(obj) => obj.to_json(),
            Value::Array(arr) => {
                let elements: Vec<String> = arr.iter().map(|v| v.to_json()).collect();
                format!("[{}]", elements.join(", "))
            }
            Value::String(s) => format!("\"{}\"", s),
            Value::Number(n) => n.clone(),
            Value::Boolean(b) => b.to_string(),
            Value::Null => "null".to_string(),
        }
    }
}

/// Represents a key-value pair in a JSON object.
#[derive(Debug, Accessors)]
pub struct PairStruct {
    key: String,
    value: Value,
}

impl PairStruct {
    /// Constructs a `PairStruct` from a pest `Pair<Rule>`.
    pub fn from_pair(pair: Pair<Rule>) -> Result<Self, JsonParseError> {
        let inner_rules: Vec<_> = pair.into_inner().map(|p| (p.as_rule(), p)).collect();

        match inner_rules.as_slice() {
            [(Rule::string, key_pair), (Rule::array, value_pair)] |
            [(Rule::string, key_pair), (Rule::object, value_pair)] |
            [(Rule::string, key_pair), (Rule::number, value_pair)] |
            [(Rule::string, key_pair), (Rule::boolean, value_pair)] |
            [(Rule::string, key_pair), (Rule::null, value_pair)]  |
            [(Rule::string, key_pair), (Rule::string, value_pair)] |
            [(Rule::string, key_pair), (Rule::value, value_pair)] => {
                Ok(PairStruct {
                    key: strip_quotes(key_pair.as_str()).to_string(),
                    value: Value::from_pair(value_pair.clone())?,
                })
            },
            _ => Err(JsonParseError::GeneralError(format!("Cannot process pair: {:?}", inner_rules))),
        }
    }

    /// Converts `PairStruct` into a JSON property string.
    pub fn to_json(&self) -> String {
        format!("\"{}\": {}", self.key, self.value.to_json())
    }

    /// Checks if the value is a JSON object.
    pub fn is_value_object(&self) -> bool {
        self.value.is_object()
    }
}

impl<'a> From<&'a PairStruct> for (&'a String, &'a Value) {
    fn from(pair: &'a PairStruct) -> Self {
        (pair.get_key(), pair.get_value())
    }
}

/// Represents a JSON object.
#[derive(Debug, Accessors)]
pub struct Object {
    pairs: Vec<PairStruct>,
}

impl Object {
    /// Constructs an `Object` from a pest `Pair<Rule>`.
    pub fn from_pair(pair: Pair<Rule>) -> Result<Self, JsonParseError> {
        let inner_rules: Vec<_> = pair.into_inner().map(|p| (p.as_rule(), p)).collect();

        match inner_rules.as_slice() {
            [] => Ok(Object { pairs: Vec::new() }),
            [(Rule::object, obj_pair)] => Object::from_pair(obj_pair.clone()),
            slice => {
                let pairs: Vec<PairStruct> = slice.iter()
                    .filter_map(|(rule, p)| {
                        if *rule == Rule::pair {
                            Some(PairStruct::from_pair(p.clone()).unwrap())
                        } else {
                            None
                        }
                    })
                    .collect();
                Ok(Object { pairs })
            }
        }
    }

    /// Converts `Object` into a JSON string representation.
    pub fn to_json(&self) -> String {
        let pairs_json: Vec<String> = self.pairs.iter().map(|pair| pair.to_json()).collect();
        format!("{{ {} }}", pairs_json.join(", "))
    }

    /// get pair value from key
    pub fn get_value_from_key(&self, key:&str) -> Option<&Value> {
        for pair in &self.pairs {
            if pair.key == key {
                return Some(&pair.value);
            }
        }

        None
    }

    /// Returns all key-value pairs as tuples.
    pub fn get_pair_tuples(&self) -> Vec<(&String, &Value)> {
        self.pairs.iter().map(|p| p.into()).collect()
    }

    /// Checks if all values are simple types (not objects).
    pub fn is_simple_type(&self) -> bool {
        for pair in &self.pairs {
            if pair.is_value_object() {
                return false;
            }
        }
        true
    }
}

impl Index<&str> for Object {
    type Output = Value;

    fn index(&self, key: &str) -> &Self::Output {
        for pair in &self.pairs {
            if pair.key == key {
                return &pair.value;
            }
        }

        panic!("Key '{}' not found in JSON object", key);
    }
}
