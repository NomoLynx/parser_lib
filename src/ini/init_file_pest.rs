use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

use crate::common::debug::*;
use crate::ini::*;

#[derive(Parser)]
#[grammar = "ini/init_file.pest"]
pub struct InitFileParser;

#[derive(Debug, Clone)]
pub struct CodeGenConfiguration {

}

impl Default for CodeGenConfiguration {
    fn default() -> Self {
        Self {  }
    }
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum InitFileError {
    MissingCase(String),
    ParsingConversionError,
    ParsingError,
    FileError,
}

/// struct File definition
#[derive(Debug, Clone)]
pub struct InitFile {
    type1 : Vec<Type1Type>,
}

/// implement File
impl InitFile {
    pub fn from_pair(pair:&Pair<Rule>, config:&mut CodeGenConfiguration) -> Result<Self, InitFileError> { 
        assert!(pair.as_rule() == Rule::file);
        let inner = pair.to_owned().into_inner();
        let pairs = inner.map(|x| (x.as_rule(), x)).collect::<Vec<_>>();
        match pairs.as_slice() {
            [all@.., (Rule::EOI, _)] |
            [all@..] => {
                let items = all.iter()
                    .map(|x| Type1Type::from_pair(&x.1, config).unwrap())
                    .collect::<Vec<_>>();
                Ok( Self { type1 : items } )
            }
        }
    }

    pub fn parse(input:&str) -> Result<Self, InitFileError> {
        let result = InitFileParser::parse(Rule::file, input);

        match result {
            Ok(mut pairs) => {
                if let Some(pair) = pairs.find(|n| n.as_str().len() == input.len()) {
                    let mut config = CodeGenConfiguration::default();
                    match Self::from_pair(&pair, &mut config) {
                        Ok(rr) => {
                            Ok(rr)
                        }
                        Err(err2) => {
                            error_string(format!("{:?}", err2));
                            Err(err2)
                        }
                    }
                }
                else {
                    error_string(format!("cannot match all string in {}", input));
                    Err(InitFileError::ParsingError)
                }
            }
            Err(err) => {
                error_string(format!("{}", err));
                Err(InitFileError::ParsingError)
            }
        }
    }

    pub fn get_items(&self) -> &Vec<Type1Type> {
        &self.type1
    }

    pub fn get_first_layer_properties(&self) -> Vec<&Property> {
        self.get_items()
            .iter()
            .filter_map(|x| x.get_property())
            .collect::<Vec<_>>()
    }

    pub fn get_property_under_section(&self, section:&str) -> Vec<&Property> {
        for item in self.get_items() {
            if let Some(section_data) = item.get_section_data() {
                if section_data.get_section().trim() == section.trim() {
                    let r = section_data.get_property();
                    let rr = r.into_iter().map(|p| p).collect::<Vec<_>>();
                    return rr;
                }
            }
        }

        vec![]
    }
}