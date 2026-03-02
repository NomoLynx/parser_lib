pub mod string_formatter_item;
pub mod formatted_string;
pub mod path_string;
pub mod datetime_format_string;
pub mod string_format_error;

use pest::iterators::Pair;
use pest_derive::Parser;
use core_utils::expr_value::ExprValue;

use core_utils::debug::*;

use self::string_formatter_item::{Formatter, StringFormatterItem};
use self::string_format_error::StringFormatError;

#[derive(Parser)]
#[grammar = "basic_type_grammar.pest"]
#[grammar = "string_format/string_format.pest"]
pub struct StringFormatParser;

pub(self) fn from_pair_template<T>(
    pair: &Pair<Rule>,
    rule: Rule,
    f: fn(Vec<(Rule, Pair<Rule>)>) -> Result<T, StringFormatError>,
) -> Result<T, StringFormatError> {
    if pair.as_rule() == rule {
        let inner = pair.to_owned().into_inner();
        let rules: Vec<(Rule, Pair<Rule>)> = inner.map(|x| (x.as_rule(), x)).collect();
        f(rules)
    } else {
        println!(
            "expected rule = {:?} which cannot match {:#?} so cannot continue in from_pair_template",
            rule, pair
        );
        Err(StringFormatError::ParsingConversionError)
    }
}

pub(self) fn from_pair_vec_template<T, T2>(
    pair: &Pair<Rule>,
    rule: Rule,
    f: fn(Pair<Rule>) -> T2,
    f2: fn(Vec<T2>) -> T,
) -> Result<T, StringFormatError> {
    if pair.as_rule() == rule {
        let inner = pair.to_owned().into_inner();
        let rules: Vec<T2> = inner.map(|x| f(x)).collect();
        Ok(f2(rules))
    } else {
        println!("{:?} cannot match {:#?}", rule, pair);
        Err(StringFormatError::ParsingConversionError)
    }
}

pub(self) fn pair_to_i32(pair: &Pair<Rule>) -> Result<i32, StringFormatError> {
    if pair.as_str().starts_with("0x") {
        i32::from_str_radix(&pair.as_str()[2..], 16)
            .map_err(|_| StringFormatError::ParsingConversionError)
    } else {
        i32::from_str_radix(pair.as_str(), 10)
            .map_err(|_| StringFormatError::ParsingConversionError)
    }
}

pub fn i64_to_string(v: i64, formatter: &StringFormatterItem) -> String {
    let width_option = formatter.get_range();
    let precision_option = formatter.get_precision();
    let data_type_option = formatter.get_formatter();
    let s = match (data_type_option, width_option, precision_option) {
        (Some(Formatter::Bin), Some(width), None) => format!("{v:width$b}", width = (width as usize), v = v),
        (Some(Formatter::Hex), Some(width), None) => format!("{v:width$X}", width = (width as usize), v = v),
        (Some(Formatter::Oct), Some(width), None) => format!("{v:width$o}", width = (width as usize), v = v),
        (Some(Formatter::ScientificFormat), Some(width), None) => {
            format!("{v:width$e}", width = (width as usize), v = v)
        }
        (Some(Formatter::Normal), Some(width), Some(precision)) => {
            format!("{v:width$.precision$}", width = (width as usize), v = v, precision = (precision as usize))
        }
        (Some(Formatter::Normal), None, Some(precision)) => {
            format!("{v:.precision$}", v = v, precision = (precision as usize))
        }
        (Some(Formatter::Normal), Some(width), None) => {
            format!("{v:width$}", width = (width as usize), v = v)
        }
        (Some(Formatter::Normal), None, None) => format!("{}", v),
        (Some(Formatter::Percentage), _, _) => i64_to_string(v * 100, formatter),
        (None, Some(width), Some(precision)) => {
            format!("{v:width$.precision$}", width = (width as usize), v = v, precision = (precision as usize))
        }
        (None, None, Some(precision)) => format!("{v:.precision$}", v = v, precision = (precision as usize)),
        (None, Some(width), None) => format!("{v:width$}", width = (width as usize)),
        (None, None, None) => format!("{v}", v = v),
        _ => {
            error_string(format!("need implement string formatter {:?}", formatter));
            todo!()
        }
    };

    s
}

pub fn u64_to_string(v: u64, formatter: &StringFormatterItem) -> String {
    let width_option = formatter.get_range();
    let precision_option = formatter.get_precision();
    let data_type_option = formatter.get_formatter();
    let s = match (data_type_option, width_option, precision_option) {
        (Some(Formatter::Bin), Some(width), None) => format!("{v:width$b}", width = (width as usize), v = v),
        (Some(Formatter::Hex), Some(width), None) => format!("{v:width$X}", width = (width as usize), v = v),
        (Some(Formatter::Oct), Some(width), None) => format!("{v:width$o}", width = (width as usize), v = v),
        (Some(Formatter::ScientificFormat), Some(width), None) => {
            format!("{v:width$e}", width = (width as usize), v = v)
        }
        (Some(Formatter::Normal), Some(width), Some(precision)) => {
            format!("{v:width$.precision$}", width = (width as usize), v = v, precision = (precision as usize))
        }
        (Some(Formatter::Normal), None, Some(precision)) => {
            format!("{v:.precision$}", v = v, precision = (precision as usize))
        }
        (Some(Formatter::Normal), Some(width), None) => {
            format!("{v:width$}", width = (width as usize), v = v)
        }
        (Some(Formatter::Normal), None, None) => format!("{}", v),
        (Some(Formatter::Percentage), _, _) => u64_to_string(v * 100, formatter),
        (None, Some(width), Some(precision)) => {
            format!("{v:width$.precision$}", width = (width as usize), v = v, precision = (precision as usize))
        }
        (None, None, Some(precision)) => format!("{v:.precision$}", v = v, precision = (precision as usize)),
        (None, Some(width), None) => format!("{v:width$}", width = (width as usize)),
        (None, None, None) => format!("{v}", v = v),
        _ => {
            error_string(format!("need implement string formatter {:?}", formatter));
            todo!()
        }
    };

    s
}

pub fn f64_to_string(v: f64, formatter: &StringFormatterItem) -> String {
    let width_option = formatter.get_range();
    let precision_option = formatter.get_precision();
    let data_type_option = formatter.get_formatter();
    let s = match (data_type_option, width_option, precision_option) {
        (Some(Formatter::ScientificFormat), Some(width), None) => {
            format!("{v:width$e}", width = (width as usize), v = v)
        }
        (Some(Formatter::Percentage), _, _) => f64_to_string(v * 100., formatter),
        (Some(Formatter::Normal), Some(width), Some(precision)) | (None, Some(width), Some(precision)) => {
            format!("{v:width$.precision$}", width = (width as usize), v = v, precision = (precision as usize))
        }
        (Some(Formatter::Normal), None, Some(precision)) | (None, None, Some(precision)) => {
            format!("{v:.precision$}", v = v, precision = (precision as usize))
        }
        (Some(Formatter::Normal), Some(width), None) | (None, Some(width), None) => {
            format!("{v:width$}", width = (width as usize))
        }
        (Some(Formatter::Normal), None, None) | (None, None, None) => format!("{v}", v = v),
        _ => todo!(),
    };

    s
}

pub(self) fn expr_value_to_string(
    v: &ExprValue,
    formatter: &StringFormatterItem,
) -> Result<String, StringFormatError> {
    match v {
        ExprValue::Int8(_)
        | ExprValue::Int16(_)
        | ExprValue::Int32(_)
        | ExprValue::Int64(_)
        | ExprValue::Int128(_)
        | ExprValue::NativeInt(_) => {
            let n = v.conv_i64()?.i64()?;
            Ok(i64_to_string(n, formatter))
        }
        ExprValue::UInt8(_)
        | ExprValue::UInt16(_)
        | ExprValue::UInt32(_)
        | ExprValue::UInt64(_)
        | ExprValue::UInt128(_)
        | ExprValue::NativeUInt(_) => {
            let n = v.conv_u64()?.u64()?;
            Ok(u64_to_string(n, formatter))
        }
        ExprValue::Float32(_) | ExprValue::Float64(_) => {
            let n = v.conv_f64()?.f64()?;
            Ok(f64_to_string(n, formatter))
        }
        ExprValue::String(s) => Ok(s.clone().unwrap_or_default()),
        ExprValue::Char(c) => Ok(c.to_string()),
        ExprValue::Boolean(b) => Ok(b.to_string()),
        ExprValue::None => Ok(String::new()),
        _ => Ok(format!("{v:?}")),
    }
}
