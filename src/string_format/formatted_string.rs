use std::fmt::Display;

use pest::{iterators::Pair, Parser};
use core_utils::debug::*;
use core_utils::expr_value::ExprValue;

use super::{expr_value_to_string, from_pair_template, from_pair_vec_template, path_string::PathString, string_format_error::StringFormatError, string_formatter_item::StringFormatterItem, Rule, StringFormatParser};

/// trait for the Pest pair to internal data structure
pub trait PestPairTrait<T> {
    fn from_pair(pair:&Pair<Rule>) -> Result<T, StringFormatError>;
}

#[derive(Debug, PartialEq)]
pub enum FormattedString { 
    Raw(RawString),
    Verbatim(VerbatimString),
    Path(PathString),
}

impl FormattedString {
    pub fn new_from_raw_string(v:RawString) -> Self {
        Self::Raw(v)
    }

    pub fn new_from_verbatim_string(v:VerbatimString) -> Self {
        Self::Verbatim(v)
    }

    pub fn process(&self, params:&Vec<ExprValue>) -> Result<String, StringFormatError> {
        match self {
            Self::Raw(n) => n.process(params),
            Self::Verbatim(n) => n.process(params),
            Self::Path(n) => n.process(params),
        }
    }

    pub fn parse(str:&String) -> Result<FormattedString, StringFormatError> {
        let result = StringFormatParser::parse(Rule::formatted_string, str.as_str());
        match result {
            Ok(mut pairs) => {
                if let Some(pair) = pairs.find(|n| n.as_str().len() == str.len()) {
                    FormattedString::from_pair(&pair)
                }
                else {
                    Err(StringFormatError::ParsingConversionError)
                }
            }
            Err(err) => {
                error_string(format!("{}", err));
                Err(StringFormatError::ParsingConversionError)
            }
        }
    }

    pub fn get_path_string(&self) -> Result<&PathString, StringFormatError> {
        match self {
            Self::Path(n) => Ok(n),
            _ => Err(StringFormatError::WrongType(format!("current type is {self:?}"))),
        }
    }
}

impl Display for FormattedString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Raw(n) => n.to_string(),
            Self::Verbatim(n) => n.to_string(),
            Self::Path(n) => n.to_string(),
        };

        write!(f, "{}", s)
    }
}

impl PestPairTrait<FormattedString> for FormattedString {
    fn from_pair(pair:&Pair<Rule>) -> Result<FormattedString, StringFormatError> {
        from_pair_template(pair, Rule::formatted_string, |rules| {
            match rules.as_slice() {
                [_, (Rule::raw_string, p), _] | 
                [(Rule::raw_string, p), _] | 
                [_, (Rule::raw_string, p)] |
                [(Rule::raw_string, p)] => 
                    Ok(Self::new_from_raw_string(RawString::from_pair(p)?)),
                [_, (Rule::verbatim_string, p), _] | 
                [(Rule::verbatim_string, p), _] | 
                [_, (Rule::verbatim_string, p)] |
                [(Rule::verbatim_string, p)] => 
                    Ok(Self::new_from_verbatim_string(VerbatimString::from_pair(p)?)),
                [_, (Rule::path_string, p), _] | 
                [(Rule::path_string, p), _] | 
                [_, (Rule::path_string, p)] |
                [(Rule::path_string, p)] => 
                    Ok(Self::Path(PathString::from_pair(p)?)),
                _ => {
                    error_string(format!("Missed case: parsing error with rules = {:?}", rules));
                    Err(StringFormatError::ParsingConversionError)
                }
            }
        })
    }
}

#[derive(Debug)]
pub struct VerbatimString {
    value : Vec<VerbatimStringContent>,
}

impl VerbatimString {
    pub fn new(v:Vec<VerbatimStringContent>) -> Self {
        Self { value: v }
    }

    pub fn process(&self, params:&Vec<ExprValue>) -> Result<String, StringFormatError> {
        let mut r = String::new();
        for n in &self.value {            
            let s = n.process(params);            
            r.push_str(s?.as_str());
        }

        Ok(r)
    }
}

impl PestPairTrait<VerbatimString> for VerbatimString {
    fn from_pair(pair:&Pair<Rule>) -> Result<VerbatimString, StringFormatError> {
        from_pair_vec_template(pair, 
            Rule::verbatim_string, 
            |p| { VerbatimStringContent::from_pair(&p).unwrap() }, 
            |vec| { VerbatimString::new(vec) })
    }
}

impl Display for VerbatimString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self.value
                            .iter()
                            .map(|x| x.to_string())
                            .collect::<Vec<_>>()
                            .concat();
        write!(f, "{}", s)
    }
}

impl PartialEq for VerbatimString {
    fn eq(&self, other: &Self) -> bool {
        self.value.len() == other.value.len() &&
        self.value == other.value
    }
}

#[derive(Debug)]
pub struct RawString {
    value : Vec<RawStringContent>,
}

impl RawString {
    pub fn new(value:Vec<RawStringContent>) -> Self {
        Self { value }
    }

    pub fn process(&self, params:&Vec<ExprValue>) -> Result<String, StringFormatError> {
        let mut r = String::new();
        for n in &self.value {            
            let s = n.process(params)?;            
            r.push_str(s.as_str());
        }

        Ok(r)
    }
}

impl Display for RawString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self.value
                            .iter()
                            .map(|x| x.to_string())
                            .collect::<Vec<_>>()
                            .concat();
        write!(f, "{}", s)
    }
}

impl PestPairTrait<RawString> for RawString {
    fn from_pair(pair:&Pair<Rule>) -> Result<RawString, StringFormatError> {
        from_pair_vec_template(pair, 
            Rule::raw_string, 
            |p| { RawStringContent::from_pair(&p).unwrap() }, 
            |vec| { RawString::new(vec) })
    }
}

impl PartialEq for RawString {
    fn eq(&self, other: &Self) -> bool {
        self.value.len() == other.value.len() &&
        self.value == other.value
    }
}

#[derive(PartialEq, Debug)]
pub struct EscapedItem {
    value : String,
}

impl EscapedItem {
    pub fn new(v:String) -> Self {
        Self { value: v }
    }

    pub fn to_original_string(&self) -> String {
        format!("\\{}", self.value)
    }

    pub fn to_char(&self) -> char {
        match self.value.as_str() {
            "n" => '\n',
            "r" => '\r',
            "t" => '\t',
            "\"" => '"',
            "'" => '\'',
            "0" => '\0',
            _ => self.value.chars().nth(0).unwrap(),  //the 1st char is the true value
        }
    }
}

impl PestPairTrait<EscapedItem> for EscapedItem {
    fn from_pair(pair:&Pair<Rule>) -> Result<EscapedItem, StringFormatError> {
        if pair.as_rule() == Rule::escaped_item {
            Ok(EscapedItem::new(pair.as_str()[1..].to_string()))
        }
        else {
            error_string(format!("Bound cannot match {:#?}", pair));
            Err(StringFormatError::ParsingConversionError)
        }
    }
}

impl Display for EscapedItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = format!("{}", self.value);
        write!(f, "{}", s)
    }
}

#[derive(Debug, PartialEq)]
pub struct VerbatimEscapeItem {
    value : String,
}

impl VerbatimEscapeItem {
    pub fn new(v:String) -> Self {
        Self { value: v }
    }

    pub fn to_original_string(&self) -> String {
        format!("\"{}", self.value)
    }

    pub fn to_char(&self) -> char {
        match self.value.as_str() {
            _ => self.value.chars().nth(0).unwrap(),  //the 1st char is the true value
        }
    }
}

impl PestPairTrait<VerbatimEscapeItem> for VerbatimEscapeItem {
    fn from_pair(pair:&Pair<Rule>) -> Result<VerbatimEscapeItem, StringFormatError> {
        if pair.as_rule() == Rule::verbatim_escape_item {
            Ok(VerbatimEscapeItem::new(pair.as_str()[1..].to_string()))
        }
        else {
            error_string(format!("rule does not match, the current pair: {:#?}", pair));
            Err(StringFormatError::ParsingConversionError)
        }
    }
}

impl Display for VerbatimEscapeItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = format!("{}", self.value);
        write!(f, "{}", s)
    }
}

#[derive(Debug)]
pub enum RawStringContent {
    Formatter(StringFormatterItem),
    PlainString(String), 
    EscapedItem(EscapedItem),
}

impl RawStringContent {
    pub fn process(&self, params:&Vec<ExprValue>) -> Result<String, StringFormatError> {
        match self {
            Self::EscapedItem(n) => Ok(n.to_char().to_string()),
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

impl Display for RawStringContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Formatter(n) => format!("{}", n),
            Self::PlainString(n) => format!("{}", n),
            Self::EscapedItem(n) => format!("{}", n),
        };

        write!(f, "{}", s)
    }
}

impl PestPairTrait<RawStringContent> for RawStringContent {
    fn from_pair(pair:&Pair<Rule>) -> Result<RawStringContent, StringFormatError> {
        from_pair_template(pair, Rule::raw_string_content, |rules| {
            match rules.as_slice() {
                [(Rule::escaped_item, p)] => Ok(Self::EscapedItem(EscapedItem::from_pair(p)?)),
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

impl PartialEq for RawStringContent {
    fn eq(&self, other: &Self) -> bool {
        let r = match (self, other) {
            (Self::Formatter(m), Self::Formatter(n)) => m==n,
            (Self::PlainString(m), Self::PlainString(n)) => m==n,
            (Self::EscapedItem(m), Self::EscapedItem(n)) => m==n,
            _ => false,
        };
        
        r
    }
}

#[derive(Debug)]
pub enum VerbatimStringContent {
    Formatter(StringFormatterItem),
    PlainString(String), 
    EscapedItem(VerbatimEscapeItem),
}

impl VerbatimStringContent {
    pub fn process(&self, params:&Vec<ExprValue>) -> Result<String, StringFormatError> {
        match self {
            Self::EscapedItem(n) => Ok(n.to_char().to_string()),
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

impl PestPairTrait<VerbatimStringContent> for VerbatimStringContent {
    fn from_pair(pair:&Pair<Rule>) -> Result<VerbatimStringContent, StringFormatError> {
        from_pair_template(pair, Rule::verbatim_string_content, |rules| {
            match rules.as_slice() {
                [(Rule::verbatim_escape_item, p)] => Ok(Self::EscapedItem(VerbatimEscapeItem::from_pair(p)?)),
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

impl PartialEq for VerbatimStringContent {
    fn eq(&self, other: &Self) -> bool {
        let r = match (self, other) {
            (Self::Formatter(m), Self::Formatter(n)) => m==n,
            (Self::PlainString(m), Self::PlainString(n)) => m==n,
            (Self::EscapedItem(m), Self::EscapedItem(n)) => m==n,
            _ => false,
        };
        
        r
    }
}

impl Display for VerbatimStringContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Formatter(n) => format!("{}", n),
            Self::PlainString(n) => format!("{}", n),
            Self::EscapedItem(n) => format!("{}", n),
        };

        write!(f, "{}", s)
    }
}
