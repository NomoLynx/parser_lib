use std::time::{SystemTime, UNIX_EPOCH};

use pest::{iterators::Pair, Parser};
use core_utils::debug::*;

use super::{formatted_string::PestPairTrait, from_pair_vec_template, pair_to_i32, string_format_error::StringFormatError, Rule, StringFormatParser};

#[derive(Debug, PartialEq, Clone)]
pub struct DateTime { 
    year: i32, 
    month: u8, 
    day: u8, 
    hour: u8, 
    minute:u8, 
    second: u8, 
    millisecond: u16, 
}

impl DateTime {
    pub fn new(year:i32, month:u8, day:u8, hour:u8, minute:u8, second:u8, millisecond:u16) -> Self {
        Self { year, month, day, hour, minute, second, millisecond }
    }

    pub fn to_string_with_format(&self, format_string:&String) -> Result<String, StringFormatError> {
        let string_format_items = DateTimeStringItems::parse(format_string)?;
        let mut r = String::default();

        let has_am_pm = string_format_items.has_am_pm();

        for item in string_format_items.value.iter() {
            let s = match item {
                DateTimeStringItem::AMPM(_s) => self.get_am_pm(),
                DateTimeStringItem::Day(n) => if n == &DayFormat::LongFormat { format!("{:02}", self.day) } else { format!("{}", self.day) },
                DateTimeStringItem::Hour(n) => if n == &HourFormat::LongFormat { format!("{:02}", self.get_hour(has_am_pm)) } else { format!("{}", self.get_hour(has_am_pm)) }
                DateTimeStringItem::Millisecond(n) => if n == &MilsecondFormat::LongFormat { format!("{:06}", self.millisecond) } else { format!("{}", self.millisecond) }
                DateTimeStringItem::Minute(n) => if n == &MinuteFormat::LongFormat { format!("{:02}", self.minute) } else { format!("{}", self.minute) }
                DateTimeStringItem::Month(n) => if n == &MonthFormat::LongFormat { format!("{:02}", self.month) } else { format!("{}", self.month) }
                DateTimeStringItem::Second(n) => if n == &SecondFormat::LongFormat { format!("{:02}", self.second) } else { format!("{}", self.second) }
                DateTimeStringItem::Splitter(n) => n.to_string(),
                DateTimeStringItem::Year(n) => if n == &YearFormat::LongFormat { format!("{}", self.year) } else { format!("{:02}", self.year % 100) }
            };

            r.push_str(s.as_str());
        }

        Ok(r)
    }

    fn get_hour(&self, has_am_pm:bool) -> u8 {
        if has_am_pm {
            self.hour % 12
        }
        else {
            self.hour
        }
    }

    fn get_am_pm(&self) -> String {
        if self.hour >=12 { "PM".to_string() }
        else { "AM".to_string() }
    }

    pub fn get_current_date_time() -> Result<Self, StringFormatError> {
        let now = SystemTime::now();
        let since_epoch = now.duration_since(UNIX_EPOCH).map_err(|_| StringFormatError::TimeError)?;
        let secs = since_epoch.as_secs();
        let millisecond = since_epoch.subsec_millis();

        let year = secs / 31_556_926 + 1970;
        let month = ((secs % 31_556_926) / 2_592_000) + 1;
        let day = ((secs % 31_556_926) % 2_592_000) / 86_400;

        let minute = ((secs % 86_400) % 3600) / 60;
        let seconds = secs % (24 * 3600) % 3600 % 60;
        let hour = (secs % (24 * 3600)) / 3600;

        Ok(Self { 
            year : year as i32,
            month : month as u8,
            day : day as u8,
            hour : hour as u8,
            minute : minute as u8,
            millisecond : millisecond as u16,
            second : seconds as u8,
        })
    }
}

impl  Default for DateTime {
    fn default() -> Self {
        Self::new(2000, 1, 1, 0, 0, 0, 0)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum YearFormat {
     LongFormat, // like 2020 
     ShortFormat, // like 20 
}

#[derive(Debug, PartialEq, Clone)]
pub enum MonthFormat { 
    LongFormat, //02 
    ShortFormat //2 as Feb 
} 

#[derive(Debug, PartialEq, Clone)]
pub enum DayFormat { 
    LongFormat, //09 
    ShortFormat //21 
}

#[derive(Debug, PartialEq, Clone)]
pub enum HourFormat { 
    LongFormat, //03 
    ShortFormat //3 
} 

#[derive(Debug, PartialEq, Clone)]
pub enum MinuteFormat { 
    LongFormat, //03 
    ShortFormat //3 
}

#[derive(Debug, PartialEq, Clone)]
pub enum SecondFormat { 
    LongFormat, //03 
    ShortFormat //3
} 

#[derive(Debug, PartialEq, Clone)]
pub enum MilsecondFormat {
    LongFormat,
    ShortFormat,
}

pub type Splitter = String;

#[derive(Debug, PartialEq, Clone)]
pub enum DateTimeStringItem {
    Year(YearFormat),
    Month(MonthFormat),
    Day(DayFormat),
    Hour(HourFormat),
    Minute(MinuteFormat),
    Second(SecondFormat),
    Millisecond(MilsecondFormat),
    AMPM(String),
    Splitter(Splitter),
}

impl DateTimeStringItem {
    pub fn is_am_pm(&self) -> bool {
        match self {
            Self::AMPM(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct DateTimeStringItems {
    value: Vec<DateTimeStringItem>,
}

impl DateTimeStringItems {
    pub fn parse(str:&String) -> Result<DateTimeStringItems, StringFormatError> {
        let result = StringFormatParser::parse(Rule::datetime_format_string, str);
        match result {
            Ok(pairs) => {
                let l = pairs.collect::<Vec<_>>();

                if let Some(pair) = l.iter().find(|n| n.as_str().len() == str.len()) {
                    Self::from_pair(&pair)
                }
                else {
                    error_string(format!("{:?}", l));
                    Err(StringFormatError::ParsingConversionError)
                }
            }
            Err(err) => {
                error_string(format!("{}", err));
                Err(StringFormatError::ParsingConversionError)
            }
        }
    }

    pub fn new(value:Vec<DateTimeStringItem>) -> Self {
        Self { value }
    }

    pub fn has_am_pm(&self) -> bool {
        self.value.iter()
            .any(|x| x.is_am_pm())
    }
}

impl PestPairTrait<DateTimeStringItems> for DateTimeStringItems {
    fn from_pair(pair:&Pair<Rule>) -> Result<DateTimeStringItems, StringFormatError> {
        from_pair_vec_template(pair, Rule::datetime_format_string, 
            |rule| { 
                match rule.as_rule() {
                    Rule::year_format => if rule.as_str().len() == 4 { DateTimeStringItem::Year(YearFormat::LongFormat) } 
                                         else { DateTimeStringItem::Year(YearFormat::ShortFormat) },
                    Rule::month_format => if rule.as_str().len() == 2 { DateTimeStringItem::Month(MonthFormat::LongFormat) } 
                                        else { DateTimeStringItem::Month(MonthFormat::ShortFormat) },
                    Rule::day_format => if rule.as_str().len() == 2 { DateTimeStringItem::Day(DayFormat::LongFormat) } 
                                        else { DateTimeStringItem::Day(DayFormat::ShortFormat) },
                    Rule::hour_format => if rule.as_str().len() == 2 { DateTimeStringItem::Hour(HourFormat::LongFormat) } 
                                        else { DateTimeStringItem::Hour(HourFormat::ShortFormat) },
                    Rule::minute_format => if rule.as_str().len() == 2 { DateTimeStringItem::Minute(MinuteFormat::LongFormat) } 
                                        else { DateTimeStringItem::Minute(MinuteFormat::ShortFormat) },
                    Rule::second_format => if rule.as_str().len() == 2 { DateTimeStringItem::Second(SecondFormat::LongFormat) } 
                                        else { DateTimeStringItem::Second(SecondFormat::ShortFormat) },
                    Rule::millisecond_format => {
                        let inner = rule.into_inner();
                        let n = pair_to_i32(&inner.last().unwrap()).unwrap();
                        if n > 6 { DateTimeStringItem::Millisecond(MilsecondFormat::LongFormat) } 
                        else { DateTimeStringItem::Millisecond(MilsecondFormat::ShortFormat) }
                    }
                    Rule::ampm_format => DateTimeStringItem::AMPM(rule.as_str().to_string()),
                    Rule::splitter => DateTimeStringItem::Splitter(rule.as_str().to_string()),
                    _ => {
                        error_string(format!("missing processing rule: {:?}", rule)); 
                        panic!("missing rule process")
                    }
                }
            }, 
            |value| { Self { value } })
    }
}

impl PartialEq for DateTimeStringItems {
    fn eq(&self, other: &Self) -> bool {
        self.value.iter()
            .zip(other.value.iter())
            .all(|(m, n)| { m == n })       
    }
}
