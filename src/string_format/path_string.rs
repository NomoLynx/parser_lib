use std::{fmt::Display, path::PathBuf};

use pest::{iterators::Pair, Parser};
use core_utils::debug::*;
use core_utils::expr_value::ExprValue;

use super::{expr_value_to_string, formatted_string::PestPairTrait, from_pair_template, string_format_error::StringFormatError, string_formatter_item::StringFormatterItem, Rule, StringFormatParser};

#[derive(Debug)]
pub struct PathString {
    separator : char,
    value : Vec<PathStringContent>,
}

impl PathString {
    pub fn process(&self, params:&Vec<ExprValue>) -> Result<String, StringFormatError> {
        let mut r = String::new();
        for n in &self.value {            
            let s = n.process(params)?; 
            r.push_str(s.as_str());
        }

        Ok(r)
    }

    pub fn new(separator:char, value:Vec<PathStringContent>) -> Self{
        Self { separator, value }
    }

    /// get pathbuff by giving arguments to the current string. 
    /// The arguemnt is like to give "abc" to replace {0} in a string
    /// if no argument, just pass Vec::default()
    pub fn to_path_buff(&self, params:&Vec<ExprValue>) -> Result<PathBuf, StringFormatError> {
        let s = self.process(params)?;
        let mut r = PathBuf::new();
        for n in s.split(self.separator) {
            r.push(n);
        }
        
        Ok(r)
    }

    pub fn get_file_name(&self) -> Option<String> {
        let s = self.process(&Vec::default()).ok()?;
        let p = PathBuf::from(s);
        let f = p.file_name()?.to_str()?.to_string();
        Some(f)
    }

    pub fn from_string<S:AsRef<str>>(input:S) -> Result<Self, StringFormatError> {
        match StringFormatParser::parse(Rule::path_string, input.as_ref()) {
            Ok(mut pairs) => {
                if let Some(pair) = pairs.find(|n| n.as_str().len() == input.as_ref().len()) {
                    Self::from_pair(&pair)
                }
                else {
                    Err(StringFormatError::ParsingConversionError)
                }
            }
            Err(_) => Err(StringFormatError::ParsingConversionError)
        }
    }

    pub fn from_string_with_separator<S:AsRef<str>>(separator:char, input:S) -> Result<Self, StringFormatError> 
        where S : AsRef<str> + std::fmt::Display 
    {
        let s = format!("#{separator}\"{input}\"");
        Self::from_string(s)
    }

    pub fn get_separator(&self) -> char {
        self.separator
    }
}

impl PestPairTrait<PathString> for PathString {
    fn from_pair(pair:&Pair<Rule>) -> Result<PathString, StringFormatError> {
        from_pair_template(pair, Rule::path_string, |rules| {
            match rules.as_slice() {
                [(Rule::path_string_content, p), others@..] => {
                    let m = '\\';                    
                    let mut v = others.iter()
                            .map(|x| PathStringContent::from_pair(&x.1).unwrap())
                            .collect::<Vec<_>>();
                    v.insert(0, PathStringContent::from_pair(p)?);
                    Ok(Self::new(m, v))
                }
                [(Rule::separator, p), others@ ..] => {
                    let m = p.as_str().chars().nth(0).unwrap();
                    let v = others.iter()
                            .map(|x| PathStringContent::from_pair(&x.1).unwrap())
                            .collect::<Vec<_>>();
                    Ok(Self::new(m, v))
                }
                _ => Err(StringFormatError::ParsingConversionError),
            }
        })        
    }
}

impl PartialEq for PathString {
    fn eq(&self, other: &Self) -> bool {
        self.value.len() == other.value.len() &&
        self.value.iter()
            .zip(other.value.iter())
            .all(|(a, b)| { a == b })
    }
}

impl Display for PathString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self.value
                            .iter()
                            .map(|x| x.to_string())
                            .collect::<Vec<_>>()
                            .concat();
        write!(f, "{}", s)
    }
}

#[derive(Debug, PartialEq)]
pub enum PathStringContent {
    Formatter(StringFormatterItem),
    PlainString(String),
}

impl PathStringContent {
    pub fn process(&self, params:&Vec<ExprValue>) -> Result<String, StringFormatError> {
        match self {
            Self::PlainString(n) => Ok(n.to_string()),
            Self::Formatter(n) => {
                let i = n.get_index() as usize;
                let param = &params[i];
                expr_value_to_string(param, n)
            }
        }
    }

    pub fn is_formatter(&self) -> bool {
        match self {
            Self::Formatter(_) => true,
            _ => false,
        }
    }
}

impl Display for PathStringContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Formatter(n) => format!("{}", n),
            Self::PlainString(n) => format!("{}", n),
        };

        write!(f, "{}", s)
    }
}

impl PestPairTrait<PathStringContent> for PathStringContent {
    fn from_pair(pair:&Pair<Rule>) -> Result<PathStringContent, StringFormatError> {
        from_pair_template(pair, Rule::path_string_content, |rules| {
             match rules.as_slice() {
                [(Rule::item, p)] => Ok(Self::Formatter(StringFormatterItem::from_pair(p)?)),
                [(Rule::plain_char_except_double_quote, p)] => Ok(Self::PlainString(p.as_str().to_string())),
                [(Rule::utf16, p)] => {
                    let v = p.as_str().to_string()[2..].to_string();
                    let u = u32::from_str_radix(&v, 16).unwrap();
                    Ok(Self::PlainString(format!("{}", char::from_u32(u).unwrap())))
                }
                [(Rule::utf32, p)] => {
                    let v = p.as_str().to_string()[4..].to_string();
                    let u = u32::from_str_radix(&v, 16).unwrap();
                    Ok(Self::PlainString(format!("{}", char::from_u32(u).unwrap())))
                }
                _ => {
                    error_string(format!("parsing error with rules = {:?}", rules));
                    Err(StringFormatError::ParsingConversionError)
                }
             }
        })       
    }
}
