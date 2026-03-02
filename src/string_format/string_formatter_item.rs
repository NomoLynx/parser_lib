use std::fmt::Display;

use core_utils::debug::*;
use pest::iterators::Pair;

use super::{from_pair_template, pair_to_i32, string_format_error::StringFormatError, Rule};

#[derive(PartialEq, Debug)]
pub struct StringFormatterItem {
    index: i32,
    range: Option<i32>,
    formatter: Option<Formatter>,
    precision: Option<i32>,
}

impl StringFormatterItem {
    pub fn from_pair(pair: &Pair<Rule>) -> Result<StringFormatterItem, StringFormatError> {
        from_pair_template(pair, Rule::item, |rules| match rules.as_slice() {
            [(Rule::index, p)] => Ok(Self {
                index: pair_to_i32(p)?,
                range: None,
                formatter: None,
                precision: None,
            }),
            [(Rule::index, p), (Rule::signed_integer, p2)] => Ok(Self {
                index: pair_to_i32(p)?,
                range: Some(pair_to_i32(p2)?),
                formatter: None,
                precision: None,
            }),
            [(Rule::index, p), (Rule::formatter, p3), (Rule::precision, p4)] => Ok(Self {
                index: pair_to_i32(p)?,
                range: None,
                formatter: Some(Formatter::from_char(p3.as_str().chars().nth(0).unwrap())),
                precision: Some(pair_to_i32(p4)?),
            }),
            [
                (Rule::index, p),
                (Rule::signed_integer, p2),
                (Rule::formatter, p3),
                (Rule::precision, p4),
            ] => Ok(Self {
                index: pair_to_i32(p)?,
                range: Some(pair_to_i32(p2)?),
                formatter: Some(Formatter::from_char(p3.as_str().chars().nth(0).unwrap())),
                precision: Some(pair_to_i32(p4)?),
            }),
            _ => {
                error_string(format!("Bound cannot match {:#?}", rules));
                Err(StringFormatError::ParsingConversionError)
            }
        })
    }

    pub fn new(index: i32, range: Option<i32>, formatter: Option<char>, precision: Option<i32>) -> Self {
        if formatter.is_none() {
            Self {
                index,
                range,
                formatter: None,
                precision,
            }
        } else {
            let c = formatter.unwrap();
            Self {
                index,
                range,
                formatter: Some(Formatter::from_char(c)),
                precision,
            }
        }
    }

    pub fn get_range(&self) -> Option<i32> {
        self.range
    }

    pub fn get_formatter(&self) -> Option<&Formatter> {
        self.formatter.as_ref()
    }

    pub fn get_precision(&self) -> Option<i32> {
        self.precision
    }

    pub fn get_index(&self) -> i32 {
        self.index
    }
}

impl Display for StringFormatterItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = format!(
            "type {:?} string format for var{}, range = {:?} with precision = {:?}",
            self.formatter, self.index, self.range, self.precision
        );
        write!(f, "{}", s)
    }
}

#[derive(PartialEq, Debug)]
pub enum Formatter {
    Normal,
    Hex,
    Oct,
    Bin,
    ScientificFormat,
    Percentage,
    Other(char),
}

impl Formatter {
    pub fn from_char(c: char) -> Self {
        match c {
            'd' | 'D' => Formatter::Normal,
            'h' | 'H' => Formatter::Hex,
            'o' | 'O' => Formatter::Oct,
            'b' | 'B' => Formatter::Bin,
            'e' | 'E' => Formatter::ScientificFormat,
            'p' | 'P' => Formatter::Percentage,
            _ => Self::Other(c),
        }
    }
}

impl Display for Formatter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Normal => "d".to_string(),
            Self::Bin => "b".to_string(),
            Self::Hex => "h".to_string(),
            Self::Oct => "o".to_string(),
            Self::Other(n) => n.to_string(),
            Self::Percentage => "p".to_string(),
            Self::ScientificFormat => "e".to_string(),
        };

        write!(f, "{}", s)
    }
}
