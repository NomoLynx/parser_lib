use pest::iterators::Pair;
use pest_derive::Parser;

use std::collections::LinkedList;
use crate::mermaid_error::*;
use rust_macro::*;

use super::{CodeGenConfiguration};

#[derive(Parser)]
#[grammar = "basic_type_grammar.pest"]
#[grammar = "mermaid_flow/flowchart.pest"]
pub(crate) struct FlowChartParser;

pub type FlowChartTitle = String;

///parse the FlowChartProgram
pub type Orientation = String;
#[derive(Debug, Clone, Accessors)]
pub struct FlowChartProgram {
    stmts: Vec<Stmt>,
    title: FlowChartTitle,
    orientation: Orientation,
}

impl FlowChartProgram {
    pub fn from_pair(pair: &Pair<Rule>,config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::program); //ensure it is Rule::program

        let mut flowchart_program = FlowChartProgram {
            title: String::new(),
            orientation: String::new(),
            stmts: vec![],
        };
        
        //fetch the Vec from the Rule::program looping into its children rules
        let list = pair
            .to_owned()
            .into_inner()
            .filter(|pair: &Pair<'_, Rule>| pair.as_rule() != Rule::EOI)
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();

        let v = list.as_slice();

        // proceed with the parser with the pair of the rules in slice
        match v {
            //rule:title+rule:orientation+rule:stmt_list
            [(Rule::title_content, p), (Rule::orientation, p1), (Rule::stmt_list, p2)] => {
                flowchart_program.title = p.as_str().to_owned();
                flowchart_program.orientation = p1.as_str().to_owned();
                flowchart_program.stmts = p2
                    .to_owned()
                    .into_inner()
                    .map(|pair| Stmt::from_pair(&pair,config))
                    .collect::<Result<Vec<Stmt>, _>>()?;
            }
            //rule:title_content+rule:stmt_list
            [(Rule::title_content, p), (Rule::stmt_list, p1)] => {
                flowchart_program.title = p.as_str().to_owned();
                flowchart_program.stmts = p1
                    .to_owned()
                    .into_inner()
                    .map(|pair| Stmt::from_pair(&pair,config))
                    .collect::<Result<Vec<Stmt>, _>>()?;
            }

            //rule:orientation+rule:stmt_list
            [(Rule::orientation, p), (Rule::stmt_list, p1)] => {
                flowchart_program.orientation = p.as_str().to_owned();
                flowchart_program.stmts = p1
                    .to_owned()
                    .into_inner()
                    .map(|pair| Stmt::from_pair(&pair,config))
                    .collect::<Result<Vec<Stmt>, _>>()?;
            }

            //rule:stmt_list
            [(Rule::stmt_list, p)] => {
                flowchart_program.stmts = p
                    .to_owned()
                    .into_inner()
                    .map(|pair| Stmt::from_pair(&pair,config))
                    .collect::<Result<Vec<Stmt>, _>>()?;
            }
            _ => return Err(MermaidError::ParsingConversionError),
        }

        Ok(flowchart_program)
    }
}

///parse the rule:statement
#[derive(Debug, Clone, EnumAccessors)]
pub enum Stmt {
    FlowChartGraph(FlowchartSubgraphs),
    //take the LinkedList is to consider the first-in, first-out in container
    FlowChartMulitpleLinks(FlowChartMulitpleLinks),
    FlowchartMultipleNodesLinks(FlowchartMultipleNodesLinks),
}

impl Stmt {
    pub fn from_pair(pair: &Pair<Rule>,config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::stmt); //ensure it is Rule::stmt

        let list = pair
            .to_owned()
            .into_inner()
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();

        let v = list.as_slice();

        match v {
            [(Rule::flowchart_subgraphs, p)] => {
                Ok(Stmt::FlowChartGraph(FlowchartSubgraphs::from_pair(&p,config)?))
            }

            [(Rule::flowchart_multiple_nodes_links, p)] => Ok(Stmt::FlowchartMultipleNodesLinks(
                FlowchartMultipleNodesLinks::from_pair(&p,config)?,
            )),

            [(Rule::flowchart_multiple_links, p)] => Ok(Stmt::FlowChartMulitpleLinks(
                FlowChartMulitpleLinks::from_pair(&p,config)?,
            )),

            _ => Err(MermaidError::ParsingConversionError),
        }
    }
}

pub type PairOfFlowChartNodes = (
    FlowChartNode,
    Option<FlowchartLinkType>,
    Option<FlowChartNode>,
);

///parse the FlowChartMulitpleLinks such as A -- text --> B -- text2 --> C
#[derive(Debug, Clone, Accessors)]
pub struct FlowChartMulitpleLinks {
    flowchart_mutiple_links: FlowChartPairNodes,
}

impl FlowChartMulitpleLinks {
    pub fn from_pair(pair: &Pair<Rule>,config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::flowchart_multiple_links); //ensure it is Rule::flowchart_multiple_links

        let list = pair
            .to_owned()
            .into_inner()
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();

        let v = list.as_slice();

        match v {
            [(Rule::flowchart_node_pair, p)] => Ok(FlowChartMulitpleLinks {
                flowchart_mutiple_links: FlowChartPairNodes::from_pair(&p,config)?,
            }),

            _ => Err(MermaidError::ParsingConversionError),
        }
    }
}

///parse the FlowChartPairNodes, for the graph:A -- text --> B -- text2 --> C
/// it will get the result: 
#[derive(Debug, Clone, Accessors)]
pub struct FlowChartPairNodes {
    flowchart_pair_nodes: LinkedList<PairOfFlowChartNodes>,
}

impl FlowChartPairNodes {
    pub fn from_pair(pair: &Pair<Rule>,config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::flowchart_node_pair); //ensure it is Rule::flowchart_node_pair

        let mut linked_list = LinkedList::new();

        let mut list = pair.to_owned().into_inner().map(|x| x).collect::<Vec<_>>();

        //take the first flowchart_node
        let mut left_flowchart_node = FlowChartNode::from_pair(&list[0],config)?;

        //remove the first one from vector and loop the rests to handle the rule：(flowchart_link_type ~ flowchart_node)*
        list.remove(0);

        if list.len() == 0 {
            linked_list.push_back((left_flowchart_node, None, None));
        } else {
            //handle the rule：(flowchart_link_type ~ flowchart_node)*
            list.chunks(2).for_each(|slice| {
                let flowchart_link_type = FlowchartLinkType::from_pair(&slice[0],config).unwrap();
                let right_flowchart_node = FlowChartNode::from_pair(&slice[1],config).unwrap();
                linked_list.push_back((
                    left_flowchart_node.clone(),
                    Some(flowchart_link_type),
                    Some(right_flowchart_node.clone()),
                ));
                left_flowchart_node = right_flowchart_node;
            })
        }
        Ok(FlowChartPairNodes {
            flowchart_pair_nodes: linked_list,
        })
    }
}

pub type PairOfFlowChartNodesLinks = (FlowChartPairNodes, Option<FlowChartPairNodes>);
///parse the FlowchartMultipleNodesLinks such as a --> b & c--> d
#[derive(Debug, Clone, Accessors)]
pub struct FlowchartMultipleNodesLinks {
    flowchart_stmts: LinkedList<PairOfFlowChartNodesLinks>,
}

impl FlowchartMultipleNodesLinks {
    pub fn from_pair(pair: &Pair<Rule>,config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::flowchart_multiple_nodes_links); //ensure it is Rule::flowchart_node_pair

        let mut linked_list = LinkedList::new();

        let mut list = pair.to_owned().into_inner().map(|x| x).collect::<Vec<_>>();

        //take the first flowchart_node
        let mut left_flowchart_node_pair = FlowChartPairNodes::from_pair(&list[0],config)?;

        //remove the first one from vector and loop the rests to handle the rule：(flowchart_link_type ~ flowchart_node)*
        list.remove(0);

        if list.len() == 0 {
            linked_list.push_back((left_flowchart_node_pair, None));
        } else {
            //handle the rule：(flowchart_link_type ~ flowchart_node)*
            list.chunks(2).for_each(|slice| {
                let right_flowchart_node_pair = FlowChartPairNodes::from_pair(&slice[1],config).unwrap();
                linked_list.push_back((
                    left_flowchart_node_pair.clone(),
                    Some(right_flowchart_node_pair.clone()),
                ));
                left_flowchart_node_pair = right_flowchart_node_pair;
            })
        }
        Ok(FlowchartMultipleNodesLinks {
            flowchart_stmts: linked_list,
        })
    }
}

///parse the FlowChartNode
#[derive(Debug, Clone)]
pub enum FlowChartNode {
    FlowchartStadiumShapedNode(FlowchartStadiumShapedNode),

    FlowchartDoubleCycleNode(FlowchartDoubleCycleNode),

    FlowchartFormCycleNode(FlowchartFormCycleNode),

    FlowchartCylindricalNode(FlowchartCylindricalNode),

    FlowchartSubroutineNode(FlowchartSubroutineNode),

    FlowchartParallelogramNode(FlowchartParallelogramNode),

    FlowchartParallelogramAltNode(FlowchartParallelogramAltNode),

    FlowchartHexagonNode(FlowchartHexagonNode),

    FlowchartTrapezoIdNode(FlowchartTrapezoIdNode),

    FlowchartTrapezoIdAltNode(FlowchartTrapezoIdAltNode),

    FlowchartRoundEdgeNode(FlowchartRoundEdgeNode),

    FlowchartAsymmetricNode(FlowchartAsymmetricNode),

    FlowchartRhombusNode(FlowchartRhombusNode),

    FlowchartPlainNode(FlowchartPlainNode),
}

impl FlowChartNode {
    pub fn from_pair(pair: &Pair<Rule>,config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::flowchart_node); //ensure it is Rule::flowchart_node

        let list = pair
            .to_owned()
            .into_inner()
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();

        let v = list.as_slice();

        match v {
            [(Rule::flowchart_stadium_shaped_node, p)] => {
                Ok(FlowChartNode::FlowchartStadiumShapedNode(
                    FlowchartStadiumShapedNode::from_pair(&p,config)?,
                ))
            }

            [(Rule::flowchart_double_cycle_node, p)] => Ok(
                FlowChartNode::FlowchartDoubleCycleNode(FlowchartDoubleCycleNode::from_pair(&p,config)?),
            ),

            [(Rule::flowchart_form_cycle_node, p)] => Ok(FlowChartNode::FlowchartFormCycleNode(
                FlowchartFormCycleNode::from_pair(&p,config)?,
            )),

            [(Rule::flowchart_cylindrical_node, p)] => Ok(FlowChartNode::FlowchartCylindricalNode(
                FlowchartCylindricalNode::from_pair(&p,config)?,
            )),

            [(Rule::flowchart_subroutine_node, p)] => Ok(FlowChartNode::FlowchartSubroutineNode(
                FlowchartSubroutineNode::from_pair(&p,config)?,
            )),

            [(Rule::flowchart_parallelogram_alt_node, p)] => {
                Ok(FlowChartNode::FlowchartParallelogramAltNode(
                    FlowchartParallelogramAltNode::from_pair(&p,config)?,
                ))
            }

            [(Rule::flowchart_hexagon_node, p)] => Ok(FlowChartNode::FlowchartHexagonNode(
                FlowchartHexagonNode::from_pair(&p,config)?,
            )),

            [(Rule::flowchart_trapezoid_node, p)] => Ok(FlowChartNode::FlowchartTrapezoIdNode(
                FlowchartTrapezoIdNode::from_pair(&p,config)?,
            )),

            [(Rule::flowchart_trapezoid_alt_node, p)] => Ok(
                FlowChartNode::FlowchartTrapezoIdAltNode(FlowchartTrapezoIdAltNode::from_pair(&p,config)?),
            ),

            [(Rule::flowchart_parallelogram_node, p)] => {
                Ok(FlowChartNode::FlowchartParallelogramNode(
                    FlowchartParallelogramNode::from_pair(&p,config)?,
                ))
            }

            [(Rule::flowchart_round_edge_node, p)] => Ok(FlowChartNode::FlowchartRoundEdgeNode(
                FlowchartRoundEdgeNode::from_pair(&p,config)?,
            )),

            [(Rule::flowchart_asymmetric_node, p)] => Ok(FlowChartNode::FlowchartAsymmetricNode(
                FlowchartAsymmetricNode::from_pair(&p,config)?,
            )),

            [(Rule::flowchart_rhombus_node, p)] => Ok(FlowChartNode::FlowchartRhombusNode(
                FlowchartRhombusNode::from_pair(&p,config)?,
            )),

            [(Rule::flowchart_plain_node, p)] => Ok(FlowChartNode::FlowchartPlainNode(
                FlowchartPlainNode::from_pair(&p,config)?,
            )),

            _ => Err(MermaidError::ParsingConversionError),
        }
    }

    pub fn get_node_name(&self) -> String {
        match self {
            Self::FlowchartAsymmetricNode(n) => n.get_node_name().clone(),
            Self::FlowchartCylindricalNode(n) => n.get_node_name().clone(),
            Self::FlowchartFormCycleNode(n) => n.get_node_name().clone(),
            Self::FlowchartPlainNode(n) => n.get_node_name().clone(),
            Self::FlowchartRoundEdgeNode(n) => n.get_node_name().clone(),
            Self::FlowchartStadiumShapedNode(n) => n.get_node_name().clone(),
            Self::FlowchartSubroutineNode(n) => n.get_node_name().clone(),
            Self::FlowchartRhombusNode(n) => n.get_node_name().clone(),
            Self::FlowchartHexagonNode(n) => n.get_node_name().clone(),
            Self::FlowchartTrapezoIdNode(n) => n.get_node_name().clone(),
            Self::FlowchartTrapezoIdAltNode(n) => n.get_node_name().clone(),
            Self::FlowchartParallelogramNode(n) => n.get_node_name().clone(),
            Self::FlowchartParallelogramAltNode(n) => n.get_node_name().clone(),
            Self::FlowchartDoubleCycleNode(n) => n.get_node_name().clone(),
        }
    }

    pub fn get_node_text(&self) -> String {
        match self {
            Self::FlowchartAsymmetricNode(n) => n.get_node_text().clone(),
            Self::FlowchartCylindricalNode(n) => n.get_node_text().clone(),
            Self::FlowchartFormCycleNode(n) => n.get_node_text().clone(),
            Self::FlowchartPlainNode(n) => n.get_node_text().clone(),
            Self::FlowchartRoundEdgeNode(n) => n.get_node_text().clone(),
            Self::FlowchartStadiumShapedNode(n) => n.get_node_text().clone(),
            Self::FlowchartSubroutineNode(n) => n.get_node_text().clone(),
            Self::FlowchartRhombusNode(n) => n.get_node_text().clone(),
            Self::FlowchartHexagonNode(n) => n.get_node_text().clone(),
            Self::FlowchartTrapezoIdNode(n) => n.get_node_text().clone(),
            Self::FlowchartTrapezoIdAltNode(n) => n.get_node_text().clone(),
            Self::FlowchartParallelogramNode(n) => n.get_node_text().clone(),
            Self::FlowchartParallelogramAltNode(n) => n.get_node_text().clone(),
            Self::FlowchartDoubleCycleNode(n) => n.get_node_text().clone(),
        }
    }
}

pub type NodeName = String;
pub type NodeText = String;

///parse the StadiumShapedNode
#[derive(Debug, Clone, Accessors)]
pub struct FlowchartStadiumShapedNode {
    node_name: NodeName,
    node_text: NodeText,
}

impl FlowchartStadiumShapedNode {
    pub fn from_pair(pair: &Pair<Rule>, _config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::flowchart_stadium_shaped_node); //ensure it is Rule::flowchart_stadium_shaped_node

        let list = pair
            .to_owned()
            .into_inner()
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();

        let v = list.as_slice();

        match v {
            [(Rule::term, p), (Rule::flowchart_stadium_shaped_node_text, p1)] => {
                Ok(FlowchartStadiumShapedNode {
                    node_name: p
                        .to_owned()
                        .into_inner()
                        .next()
                        .unwrap()
                        .as_str()
                        .to_owned(),
                    node_text: p1.as_str().to_owned(),
                })
            }

            _ => Err(MermaidError::ParsingConversionError),
        }
    }
}

///parse the PlainNode
#[derive(Debug, Clone, Accessors)]
pub struct FlowchartPlainNode {
    node_name: NodeName,
    node_text: NodeText,
}

impl FlowchartPlainNode {
    pub fn from_pair(pair: &Pair<Rule>, _config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::flowchart_plain_node); //ensure it is Rule::flowchart_plain_node

        let list = pair
            .to_owned()
            .into_inner()
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();

        let v = list.as_slice();

        match v {
            [(Rule::term, p), (Rule::flowchart_plain_node_text, p1)] => Ok(FlowchartPlainNode {
                node_name: p
                    .to_owned()
                    .into_inner()
                    .next()
                    .unwrap()
                    .as_str()
                    .to_owned(),
                node_text: p1.as_str().to_owned(),
            }),

            [(Rule::term, p)] => Ok(FlowchartPlainNode {
                node_name: p
                    .to_owned()
                    .into_inner()
                    .next()
                    .unwrap()
                    .as_str()
                    .to_owned(),
                node_text: String::new(),
            }),

            _ => Err(MermaidError::ParsingConversionError),
        }
    }
}

///parse the RoundEdgeNode
#[derive(Debug, Clone, Accessors)]
pub struct FlowchartRoundEdgeNode {
    node_name: NodeName,
    node_text: NodeText,
}

impl FlowchartRoundEdgeNode {
    pub fn from_pair(pair: &Pair<Rule>, _config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::flowchart_round_edge_node); //ensure it is Rule::flowchart_round_edge_node

        let list = pair
            .to_owned()
            .into_inner()
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();

        let v = list.as_slice();

        match v {
            [(Rule::term, p), (Rule::flowchart_round_edge_node_text, p1)] => {
                Ok(FlowchartRoundEdgeNode {
                    node_name: p
                        .to_owned()
                        .into_inner()
                        .next()
                        .unwrap()
                        .as_str()
                        .to_owned(),
                    node_text: p1.as_str().to_owned(),
                })
            }

            _ => Err(MermaidError::ParsingConversionError),
        }
    }
}

///parse the FlowchartSubroutineNode
#[derive(Debug, Clone, Accessors)]
pub struct FlowchartSubroutineNode {
    node_name: NodeName,
    node_text: NodeText,
}

impl FlowchartSubroutineNode {
    pub fn from_pair(pair: &Pair<Rule>, _config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::flowchart_subroutine_node); //ensure it is Rule::flowchart_subroutine_node

        let list = pair
            .to_owned()
            .into_inner()
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();

        let v = list.as_slice();

        match v {
            [(Rule::term, p), (Rule::flowchart_subroutine_node_text, p1)] => {
                Ok(FlowchartSubroutineNode {
                    node_name: p
                        .to_owned()
                        .into_inner()
                        .next()
                        .unwrap()
                        .as_str()
                        .to_owned(),
                    node_text: p1.as_str().to_owned(),
                })
            }

            _ => Err(MermaidError::ParsingConversionError),
        }
    }
}

///parse the FlowchartCylindricalNode
#[derive(Debug, Clone, Accessors)]
pub struct FlowchartCylindricalNode {
    node_name: NodeName,
    node_text: NodeText,
}

impl FlowchartCylindricalNode {
    pub fn from_pair(pair: &Pair<Rule>, _config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::flowchart_cylindrical_node); //ensure it is Rule::flowchart_cylindrical_node

        let list = pair
            .to_owned()
            .into_inner()
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();

        let v = list.as_slice();

        match v {
            [(Rule::term, p), (Rule::flowchart_cylindrical_node_text, p1)] => {
                Ok(FlowchartCylindricalNode {
                    node_name: p
                        .to_owned()
                        .into_inner()
                        .next()
                        .unwrap()
                        .as_str()
                        .to_owned(),
                    node_text: p1.as_str().to_owned(),
                })
            }

            _ => Err(MermaidError::ParsingConversionError),
        }
    }
}

///parse the FlowchartFormCycleNode
#[derive(Debug, Clone, Accessors)]
pub struct FlowchartFormCycleNode {
    node_name: NodeName,
    node_text: NodeText,
}

impl FlowchartFormCycleNode {
    pub fn from_pair(pair: &Pair<Rule>, _config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::flowchart_form_cycle_node); //ensure it is Rule::flowchart_form_cycle_node

        let list = pair
            .to_owned()
            .into_inner()
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();

        let v = list.as_slice();

        match v {
            [(Rule::term, p), (Rule::flowchart_form_cycle_node_text, p1)] => {
                Ok(FlowchartFormCycleNode {
                    node_name: p
                        .to_owned()
                        .into_inner()
                        .next()
                        .unwrap()
                        .as_str()
                        .to_owned(),
                    node_text: p1.as_str().to_owned(),
                })
            }

            _ => Err(MermaidError::ParsingConversionError),
        }
    }
}

///parse the FlowchartAsymmetricNode
#[derive(Debug, Clone, Accessors)]
pub struct FlowchartAsymmetricNode {
    node_name: NodeName,
    node_text: NodeText,
}

impl FlowchartAsymmetricNode {
    pub fn from_pair(pair: &Pair<Rule>, _config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::flowchart_asymmetric_node); //ensure it is Rule::flowchart_asymmetric_node

        let list = pair
            .to_owned()
            .into_inner()
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();

        let v = list.as_slice();

        match v {
            [(Rule::term, p), (Rule::flowchart_asymmetric_node_text, p1)] => {
                Ok(FlowchartAsymmetricNode {
                    node_name: p
                        .to_owned()
                        .into_inner()
                        .next()
                        .unwrap()
                        .as_str()
                        .to_owned(),
                    node_text: p1.as_str().to_owned(),
                })
            }

            _ => Err(MermaidError::ParsingConversionError),
        }
    }
}

///parse the FlowchartRhombusNode
#[derive(Debug, Clone, Accessors)]
pub struct FlowchartRhombusNode {
    node_name: NodeName,
    node_text: NodeText,
}

impl FlowchartRhombusNode {
    pub fn from_pair(pair: &Pair<Rule>, _config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::flowchart_rhombus_node); //ensure it is Rule::flowchart_rhombus_node

        let list = pair
            .to_owned()
            .into_inner()
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();

        let v = list.as_slice();

        match v {
            [(Rule::term, p), (Rule::flowchart_rhombus_node_text, p1)] => {
                Ok(FlowchartRhombusNode {
                    node_name: p
                        .to_owned()
                        .into_inner()
                        .next()
                        .unwrap()
                        .as_str()
                        .to_owned(),
                    node_text: p1.as_str().to_owned(),
                })
            }

            _ => Err(MermaidError::ParsingConversionError),
        }
    }
}

///parse the FlowchartHexagonNode
#[derive(Debug, Clone, Accessors)]
pub struct FlowchartHexagonNode {
    node_name: NodeName,
    node_text: NodeText,
}

impl FlowchartHexagonNode {
    pub fn from_pair(pair: &Pair<Rule>, _config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::flowchart_hexagon_node); //ensure it is Rule::flowchart_hexagon_node

        let list = pair
            .to_owned()
            .into_inner()
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();

        let v = list.as_slice();

        match v {
            [(Rule::term, p), (Rule::flowchart_hexagon_node_text, p1)] => {
                Ok(FlowchartHexagonNode {
                    node_name: p
                        .to_owned()
                        .into_inner()
                        .next()
                        .unwrap()
                        .as_str()
                        .to_owned(),
                    node_text: p1.as_str().to_owned(),
                })
            }

            _ => Err(MermaidError::ParsingConversionError),
        }
    }
}

///parse the FlowchartParallelogramNode
#[derive(Debug, Clone, Accessors)]
pub struct FlowchartParallelogramNode {
    node_name: NodeName,
    node_text: NodeText,
}

impl FlowchartParallelogramNode {
    pub fn from_pair(pair: &Pair<Rule>, _config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::flowchart_parallelogram_node); //ensure it is Rule::flowchart_parallelogram_node

        let list = pair
            .to_owned()
            .into_inner()
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();

        let v = list.as_slice();

        match v {
            [(Rule::term, p), (Rule::flowchart_parallelogram_node_text, p1)] => {
                Ok(FlowchartParallelogramNode {
                    node_name: p
                        .to_owned()
                        .into_inner()
                        .next()
                        .unwrap()
                        .as_str()
                        .to_owned(),
                    node_text: p1.as_str().to_owned(),
                })
            }

            _ => Err(MermaidError::ParsingConversionError),
        }
    }
}

///parse the FlowchartParallelogramAltNode
#[derive(Debug, Clone, Accessors)]
pub struct FlowchartParallelogramAltNode {
    node_name: NodeName,
    node_text: NodeText,
}

impl FlowchartParallelogramAltNode {
    pub fn from_pair(pair: &Pair<Rule>, _config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::flowchart_parallelogram_node); //ensure it is Rule::flowchart_parallelogram_node

        let list = pair
            .to_owned()
            .into_inner()
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();

        let v = list.as_slice();

        match v {
            [(Rule::term, p), (Rule::flowchart_parallelogram_alt_node_text, p1)] => {
                Ok(FlowchartParallelogramAltNode {
                    node_name: p
                        .to_owned()
                        .into_inner()
                        .next()
                        .unwrap()
                        .as_str()
                        .to_owned(),
                    node_text: p1.as_str().to_owned(),
                })
            }

            _ => Err(MermaidError::ParsingConversionError),
        }
    }
}

///parse the FlowchartTrapezoIdNode
#[derive(Debug, Clone, Accessors)]
pub struct FlowchartTrapezoIdNode {
    node_name: NodeName,
    node_text: NodeText,
}

impl FlowchartTrapezoIdNode {
    pub fn from_pair(pair: &Pair<Rule>, _config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::flowchart_trapezoid_node); //ensure it is Rule::flowchart_trapezoid_node

        let list = pair
            .to_owned()
            .into_inner()
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();

        let v = list.as_slice();

        match v {
            [(Rule::term, p), (Rule::flowchart_trapezoid_node_text, p1)] => {
                Ok(FlowchartTrapezoIdNode {
                    node_name: p
                        .to_owned()
                        .into_inner()
                        .next()
                        .unwrap()
                        .as_str()
                        .to_owned(),
                    node_text: p1.as_str().to_owned(),
                })
            }

            _ => Err(MermaidError::ParsingConversionError),
        }
    }
}

///parse the FlowchartTrapezoIdAltNode
#[derive(Debug, Clone, Accessors)]
pub struct FlowchartTrapezoIdAltNode {
    node_name: NodeName,
    node_text: NodeText,
}

impl FlowchartTrapezoIdAltNode {
    pub fn from_pair(pair: &Pair<Rule>, _config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::flowchart_trapezoid_alt_node); //ensure it is Rule::flowchart_trapezoid_alt_node

        let list = pair
            .to_owned()
            .into_inner()
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();

        let v = list.as_slice();

        match v {
            [(Rule::term, p), (Rule::flowchart_trapezoid_alt_node_text, p1)] => {
                Ok(FlowchartTrapezoIdAltNode {
                    node_name: p
                        .to_owned()
                        .into_inner()
                        .next()
                        .unwrap()
                        .as_str()
                        .to_owned(),
                    node_text: p1.as_str().to_owned(),
                })
            }

            _ => Err(MermaidError::ParsingConversionError),
        }
    }
}

///parse the FlowchartDoubleCycleNode
#[derive(Debug, Clone, Accessors)]
pub struct FlowchartDoubleCycleNode {
    node_name: NodeName,
    node_text: NodeText,
}

impl FlowchartDoubleCycleNode {
    pub fn from_pair(pair: &Pair<Rule>, _config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::flowchart_double_cycle_node); //ensure it is Rule::flowchart_double_cycle_node

        let list = pair
            .to_owned()
            .into_inner()
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();

        let v = list.as_slice();

        match v {
            [(Rule::term, p), (Rule::flowchart_double_cycle_node_text, p1)] => {
                Ok(FlowchartDoubleCycleNode {
                    node_name: p
                        .to_owned()
                        .into_inner()
                        .next()
                        .unwrap()
                        .as_str()
                        .to_owned(),
                    node_text: p1.as_str().to_owned(),
                })
            }

            _ => Err(MermaidError::ParsingConversionError),
        }
    }
}

pub type SubGraphTitle = String;
///parse the FlowchartSubgraphs
#[derive(Debug, Clone, Accessors)]
pub struct FlowchartSubgraphs {
    subgraph_title: SubGraphTitle,
    stmts: Vec<Stmt>,
}

impl FlowchartSubgraphs {
    pub fn from_pair(pair: &Pair<Rule>, config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::flowchart_subgraphs); //ensure it is Rule::flowchart_subgraphs

        let list = pair
            .to_owned()
            .into_inner()
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();

        let v = list.as_slice();

        match v {
            [(Rule::flowchart_subgraphs_title, p), (Rule::stmt_list, p1)] => {
                Ok(FlowchartSubgraphs {
                    subgraph_title: p.as_str().to_owned(),
                    stmts: p1
                        .to_owned()
                        .into_inner()
                        .map(|pair| Stmt::from_pair(&pair, config))
                        .collect::<Result<Vec<Stmt>, _>>()?,
                })
            }

            _ => Err(MermaidError::ParsingConversionError),
        }
    }
}

///parse of FlowchartLinkType
#[derive(Debug, Clone)]
pub enum FlowchartLinkType {
    FlowchartOpenlink(FlowchartOpenlink),
    FlowchartDottedlink(FlowchartDottedlink),
    FlowchartThicklink(FlowchartThicklink),
    FlowchartInvisiblelink,
    FlowchartCycleEdge,
    FlowchartCrossEdge,
    FlowchartArrowEdge,
    FlowchartRowHeadlink(FlowchartRowHeadlink),
}

impl FlowchartLinkType {
    pub fn from_pair(pair: &Pair<Rule>,config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::flowchart_link_type); //ensure it is Rule::flowchart_link_type

        let list = pair
            .to_owned()
            .into_inner()
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();

        let v = list.as_slice();

        match v {
            [(Rule::flowchart_open_link, p)] => Ok(FlowchartLinkType::FlowchartOpenlink(
                FlowchartOpenlink::from_pair(&p,config)?,
            )),

            [(Rule::flowchart_dotted_link, p)] => Ok(FlowchartLinkType::FlowchartDottedlink(
                FlowchartDottedlink::from_pair(&p,config)?,
            )),

            [(Rule::flowchart_thick_link, p)] => Ok(FlowchartLinkType::FlowchartThicklink(
                FlowchartThicklink::from_pair(&p,config)?,
            )),

            [(Rule::flowchart_row_head_link, p)] => Ok(FlowchartLinkType::FlowchartRowHeadlink(
                FlowchartRowHeadlink::from_pair(&p,config)?,
            )),

            [(Rule::flowchart_invisible_link, _p)] => Ok(FlowchartLinkType::FlowchartInvisiblelink),

            [(Rule::flowchart_cycle_edge, _p)] => Ok(FlowchartLinkType::FlowchartCycleEdge),

            [(Rule::flowchart_cross_edge, _p)] => Ok(FlowchartLinkType::FlowchartCrossEdge),

            [(Rule::flowchart_arrow_edge, _p)] => Ok(FlowchartLinkType::FlowchartArrowEdge),

            _ => Err(MermaidError::ParsingConversionError),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            FlowchartLinkType::FlowchartOpenlink(_) => "open_link",
            FlowchartLinkType::FlowchartDottedlink(_) => "dotted_link",
            FlowchartLinkType::FlowchartThicklink(_) => "thick_link",
            FlowchartLinkType::FlowchartInvisiblelink => "invisible_link",
            FlowchartLinkType::FlowchartCycleEdge => "cycle_edge",
            FlowchartLinkType::FlowchartCrossEdge => "cross_edge",
            FlowchartLinkType::FlowchartArrowEdge => "arrow_edge",
            FlowchartLinkType::FlowchartRowHeadlink(_) => "rowhead_link",
        }
    }
}

///parse of the FlowchartRowHeadlink
#[derive(Debug, Clone)]
pub enum FlowchartRowHeadlink {
    FlowchartRowHeadVericalLink(String),
    FlowchartRowHeadHorizontalLink(String),
}

impl FlowchartRowHeadlink {
    pub fn from_pair(pair: &Pair<Rule>, _config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::flowchart_row_head_link); //ensure it is Rule::flowchart_row_head_link

        let list = pair
            .to_owned()
            .into_inner()
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();

        //if rule:flowchart_row_head_link does not have the inner rule, means, it is pure link as "-->"
        if list.len() == 0 {
            return Ok(FlowchartRowHeadlink::FlowchartRowHeadHorizontalLink(
                "".to_owned(),
            ));
        }

        // if rule:flowchart_row_head_link has inner rule, then proceed with the link text
        let v = list.as_slice();

        match v {
            [(Rule::flowchart_row_head_verical_link_text, p)] => Ok(
                FlowchartRowHeadlink::FlowchartRowHeadVericalLink(p.as_str().to_owned()),
            ),

            [(Rule::flowchart_row_head_horizontal_link_text, p)] => Ok(
                FlowchartRowHeadlink::FlowchartRowHeadHorizontalLink(p.as_str().to_owned()),
            ),
            _ => Err(MermaidError::ParsingConversionError),
        }
    }
}

///parse the FlowchartOpenlink
#[derive(Debug, Clone)]
pub enum FlowchartOpenlink {
    FlowchartOpenVericalLink(String),
    FlowchartOpenHorizontalLink(String),
}

impl FlowchartOpenlink {
    pub fn from_pair(pair: &Pair<Rule>, _config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::flowchart_open_link); //ensure it is Rule::flowchart_open_link

        let list = pair
            .to_owned()
            .into_inner()
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();
        //if rule:flowchart_open_link does not have the inner rule, means, it is pure link as "---"
        if list.len() == 0 {
            return Ok(FlowchartOpenlink::FlowchartOpenHorizontalLink(
                "".to_owned(),
            ));
        }

        // if rule:flowchart_open_link has inner rule, then proceed with the link text
        let v = list.as_slice();

        match v {
            [(Rule::flowchart_open_Verical_link_text, p)] => Ok(
                FlowchartOpenlink::FlowchartOpenVericalLink(p.as_str().to_owned()),
            ),

            [(Rule::flowchart_open_horizontal_link_text, p)] => Ok(
                FlowchartOpenlink::FlowchartOpenHorizontalLink(p.as_str().to_owned()),
            ),
            _ => Err(MermaidError::ParsingConversionError),
        }
    }
}

///parse the FlowchartDottedlink
#[derive(Debug, Clone, Accessors)]
pub struct FlowchartDottedlink {
    flowchart_dotted_link_text: String,
}

impl FlowchartDottedlink {
    pub fn from_pair(pair: &Pair<Rule>, _config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::flowchart_dotted_link); //ensure it is Rule::flowchart_dotted_link

        let list = pair
            .to_owned()
            .into_inner()
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();

        //if rule:flowchart_dotted_link does not have the inner rule, means, it is pure link as "-.->"
        if list.len() == 0 {
            return Ok(FlowchartDottedlink {
                flowchart_dotted_link_text: String::new(),
            });
        }

        // if rule:flowchart_dotted_link has inner rule, then proceed with the link text

        let v = list.as_slice();

        match v {
            [(Rule::flowchart_dotted_link_text, p)] => Ok(FlowchartDottedlink {
                flowchart_dotted_link_text: p.as_str().to_owned(),
            }),
            _ => Err(MermaidError::ParsingConversionError),
        }
    }
}

///parse the FlowchartThicklink
#[derive(Debug, Clone, Accessors)]
pub struct FlowchartThicklink {
    flowchart_thick_link_text: String,
}

impl FlowchartThicklink {
    pub fn from_pair(pair: &Pair<Rule>, _config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::flowchart_thick_link); //ensure it is Rule::flowchart_thick_link

        let list = pair
            .to_owned()
            .into_inner()
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();

        //if rule:flowchart_thick_link does not have the inner rule, means, it is pure link as "==>"
        if list.len() == 0 {
            return Ok(FlowchartThicklink {
                flowchart_thick_link_text: String::new(),
            });
        }

        // if rule:flowchart_thick_link has inner rule, then proceed with the link text

        let v = list.as_slice();

        match v {
            [(Rule::flowchart_thick_link_text, p)] => Ok(FlowchartThicklink {
                flowchart_thick_link_text: p.as_str().to_owned(),
            }),
            _ => Err(MermaidError::ParsingConversionError),
        }
    }
}
