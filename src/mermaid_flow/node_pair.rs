use rust_macro::*;

use crate::mermaid_flow::{FlowchartLinkType, PairOfFlowChartNodes};

#[derive(Debug, Clone, Accessors)]
pub struct NodePair {
    from : String, 
    to : Option<String>,
    link_type: Option<FlowchartLinkType>,
}

impl From <&PairOfFlowChartNodes> for NodePair {
    fn from(pair: &PairOfFlowChartNodes) -> Self {
        let (from, link_type, to) = pair;
        NodePair {
            from: from.get_node_name().to_string(),
            to: to.as_ref().map(|n| n.get_node_name().to_string()),
            link_type: link_type.clone(),
        }
    }
}