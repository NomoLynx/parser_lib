use pest::iterators::Pair;
use rust_macro::*;

use crate::ini::*;

pub type Section = String;

/// struct SectionData definition
#[derive(Debug, Clone, Accessors)]
pub struct SectionData {
    section : Section,
	property : Vec<Property>,
    location : Location
}

/// implement SectionData
impl SectionData {
    pub fn from_pair(pair:&Pair<Rule>, config:&mut CodeGenConfiguration) -> Result<Self, InitFileError> { 
        assert!(pair.as_rule() == Rule::section_data);
        let inner = pair.to_owned().into_inner();
        let pairs = inner.map(|x| (x.as_rule(), x)).collect::<Vec<_>>();
        match pairs.as_slice() {
            [(Rule::section, p0), properties@ ..] => {
                let props = properties.iter().map(|x| Property::from_pair(&x.1, config).unwrap() ).collect::<Vec<_>>();
                let section_name = p0.clone().into_inner().next().unwrap().as_str();
                let s = section_name.to_string();
                Ok(Self { section : s, property : props, location : pair.into() })
            }
            _ => Err(InitFileError::MissingCase(format!("Missed case in SectionData {pairs:?}"))),
        }
    }
}