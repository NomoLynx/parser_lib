pub mod state_diagrams_pest;

use pest::Parser;
pub use state_diagrams_pest::*;
pub use crate::common::debug::*;

use crate::{common::read_file_content, mermaid_flow::MermaidError};

pub struct CodeGenConfiguration;

///parse the git_graph
pub fn parse_state_graph(input: &str) -> Result<StateGraphProgram, MermaidError> {

    let mut config=CodeGenConfiguration;

    let mut pairs = StateGraphParser::parse(Rule::program, input)
        .map_err(|e| MermaidError::get_location_from_pest_input_location(e.line(), &e.line_col))?;
    if let Some(pair) = pairs.find(|n| n.as_str().len() == input.len()) {
        let prog_r = StateGraphProgram::from_pair(&pair,&mut config);
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

/// Parse the state diagram from a file
pub fn parse_state_from_file(file_path: &str) -> Result<StateGraphProgram, MermaidError> {
    let input = read_file_content(file_path)
        .map_err(|_| MermaidError::FileNotFound(file_path.to_string()))?;
    parse_state_graph(&input)
}