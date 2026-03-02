use pest::iterators::Pair;
use rust_macro::*;

use crate::ini::*;

pub type Name = String;
pub type Value = String;

/// struct Property definition
#[derive(Debug, Clone, Accessors)]
pub struct Property {
    name : Name,
	value : Value,
    location : Location
}

/// implement Property
impl Property {
    pub fn from_pair(pair:&Pair<Rule>, _config:&mut CodeGenConfiguration) -> Result<Self, InitFileError> { 
        assert!(pair.as_rule() == Rule::property);
        let inner = pair.to_owned().into_inner();
        let pairs = inner.map(|x| (x.as_rule(), x)).collect::<Vec<_>>();
        match pairs.as_slice() {
            [(Rule::name, p0), (Rule::value, p1), (Rule::EOI, _)] |
            [(Rule::name, p0), (Rule::value, p1)] => {
                let r = Self { 
                    name : p0.as_str().trim().to_string(), 
                    value : p1.as_str().trim().to_string(), 
                    location : pair.into() 
                };
                
                Ok(r)
            }
            _ => Err(InitFileError::MissingCase(format!("Missed case in Property {pairs:?}"))),
        }
    }

    pub fn to_tuple(&self) -> (String, String) {
        self.into()
    }
}

impl From<&Property> for (String, String) {
    fn from(prop: &Property) -> Self {
        (prop.get_name().to_string(), prop.get_value().to_string())
    }
}