pub mod packet_pest;

pub use packet_pest::*;

use pest::Parser;

use crate::common::*;
use crate::mermaid_error::*;

pub fn parse(path_str: &str) -> Result<PacketSection, MermaidError> {
    let str = if !file_exists(path_str) {
        let err_str = format!("packet file at '{}' cannot be found", path_str);
        return Err(MermaidError::FileNotFound(err_str));
    }
    else {
        read_file_to_string(path_str)
    };

    parse_str(&str)
}

pub fn parse_str(input: &str) -> Result<PacketSection, MermaidError> {
    let result = PackatFileParser::parse(Rule::packet_diagram, input);
    
    match result {
        Ok(mut pairs) => {
            if let Some(pair) = pairs.find(|n| n.as_str().len() == input.len()) {
                match PacketSection::from_pair(&pair) {
                    Some(rr) => {
                        Ok(rr)
                    }
                    None => {
                        Err(MermaidError::InvalidPacketDefinition)
                    }
                }
            }
            else {
                Err(MermaidError::GeneralError(format!("Parsing error, doesn't match all input")))
            }
        }
        Err(err) => {
            Err(MermaidError::GeneralError(format!("Parsing error: {}", err)))
        }
    }
}