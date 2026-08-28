use std::fmt::Display;

use crate::mermaid_flow::{FlowChartProgram, parse_flowchart};
use crate::mermaid_packet::{PacketSection, parse_str};
use crate::mermaid_sequence::{SequenceProgram, parse_sequence};
use crate::mermaid_state::{StateGraphProgram, parse_state_graph};

/// The `MermaidType` enum represents the different types of Mermaid diagrams 
/// that can be parsed and processed. It includes variants for flowcharts, 
/// packet diagrams, sequence diagrams, and state diagrams. 
/// Each variant holds the corresponding parsed program structure.
pub enum MermaidType {
    Flow(FlowChartProgram),
    Packet(PacketSection),
    Sequence(SequenceProgram),
    State(StateGraphProgram),
}

impl MermaidType {

    /// the content is the mermaid type content, use the parser to parse the content and 
    /// return the mermaid type, if the content is not valid, return None
    pub fn get_mermaid_type_from_string_content(content: &str) -> Option<MermaidType> {
        if let Ok(v) = parse_flowchart(content) {
            return Some(MermaidType::Flow(v));
        } else if let Ok(v) = parse_str(content) {
            return Some(MermaidType::Packet(v));
        } else if let Ok(v) = parse_sequence(content) {
            return Some(MermaidType::Sequence(v));
        } else if let Ok(v) = parse_state_graph(content) {
            return Some(MermaidType::State(v));
        } else {
            return None;
        }
    }
}

impl Display for MermaidType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MermaidType::Flow(_) => write!(f, "flow"),
            MermaidType::Packet(_) => write!(f, "packet"),
            MermaidType::Sequence(_) => write!(f, "sequence"),
            MermaidType::State(_) => write!(f, "state"),
        }
    }
}