use crate::{common::debug::*, mermaid_flow::MermaidError};

pub mod sequence_pest;

use pest::Parser;
pub use sequence_pest::*;

pub struct CodeGenConfiguration;

pub fn parse_sequence(input: &str) -> Result<SequenceProgram, MermaidError> {

    let mut config=CodeGenConfiguration;

    let mut pairs = SequenceParser::parse(Rule::program, input)
        .map_err(|e| MermaidError::get_location_from_pest_input_location(e.line(), &e.line_col))?;
    if let Some(pair) = pairs.find(|n| n.as_str().len() == input.len()) {
        let prog_r = SequenceProgram::from_pair(&pair,&mut config);
        if prog_r.is_err() {
            error_str("cannot get program from Rule::START");
            Err(MermaidError::ParsingConversionError)
        } else {
            let prog = prog_r.unwrap();
            Ok(prog)
        }
    } else {
        error_string(format!(
            "Error: {} at {}",
            "does not catch all string",
            input.to_owned()
        ));
        debug_string(format!("input: {}\r\nParsed: {:#?}", input, pairs));
        let count = pairs.count();
        debug_string(format!("Pairs count = {count}\r\n"));
        Err(MermaidError::ParsingConversionError)
    }
}

pub fn parse_sequence_from_file(file_path: &str) -> Result<SequenceProgram, MermaidError> {
    let input = std::fs::read_to_string(file_path)
        .map_err(|_| MermaidError::FileNotFound(file_path.to_string()))?;
    parse_sequence(&input)
}