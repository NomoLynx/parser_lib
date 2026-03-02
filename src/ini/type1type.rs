use pest::iterators::Pair;
use rust_macro::GenIsEnumVariant;

use crate::ini::*;

/// enum Type1Type definition
#[derive(Debug, Clone, GenIsEnumVariant)]
pub enum Type1Type {
    Property( Property ),
	SectionData( SectionData ),
}

/// implement Type1Type
impl Type1Type {
    pub fn from_pair(pair:&Pair<Rule>, config:&mut CodeGenConfiguration) -> Result<Self, InitFileError> { 
        match pair.as_rule() {
            Rule::property => {
                Ok(Type1Type::Property(Property::from_pair(pair, config).unwrap()))
            }
            Rule::section_data => {
                Ok(Type1Type::SectionData(SectionData::from_pair(pair, config).unwrap()))
            }
            _ => Err(InitFileError::MissingCase(format!("Missed case in Type1Type {pair:?}")))
        }
    }

    pub fn get_property(&self) -> Option<&Property> {
        match self {
            Self::Property(n) => Some(n),
            _ => None,
        }
    }
    
    pub fn get_section_data(&self) -> Option<&SectionData> {
        match self {
            Self::SectionData(n) => Some(n),
            _ => None,
        }
    }
}
