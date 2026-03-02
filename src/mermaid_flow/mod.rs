pub mod flowchart_pest;
pub mod node_pair;

pub use flowchart_pest::*;
use pest::Parser;
use crate::common::*;
pub use crate::mermaid_error::*;
pub use node_pair::*;

pub struct CodeGenConfiguration;

pub fn parse_flowchart_from_path(path:&str) -> Result<FlowChartProgram, MermaidError> {
    let content = read_file_content(path)
                            .map_err(|_| MermaidError::FileError)?;
    parse_flowchart(&content)
}

///parse the flowchart
pub fn parse_flowchart(input: &str) -> Result<FlowChartProgram, MermaidError> {

    let mut config=CodeGenConfiguration;

    let mut pairs = FlowChartParser::parse(Rule::program, input)
        .map_err(|e| MermaidError::get_location_from_pest_input_location(e.line(), &e.line_col))?;
    
    if let Some(pair) = pairs.find(|n| n.as_str().len() == input.len()) {
        let prog_r = FlowChartProgram::from_pair(&pair,&mut config);
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