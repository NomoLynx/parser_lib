use crate::{common::*, mermaid_error::*};
use super::CodeGenConfiguration;
use pest::iterators::Pair;
use pest_derive::Parser;

use rust_macro::*;

#[derive(Parser)]
#[grammar = "basic_type_grammar.pest"]
#[grammar = "mermaid_state/state_diagrams.pest"]
pub(crate) struct StateGraphParser;

pub type StateDiagramTitle = String;

///parse the StateGraphProgram
pub type Orientation = String;
pub type Version = String;
#[derive(Debug, Clone)]
pub struct StateGraphProgram {
    pub stmts: Vec<Stmt>,
    pub title: StateDiagramTitle,
    pub version: Version,
}

impl StateGraphProgram {
    pub fn from_pair(pair: &Pair<Rule>,config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::program); //ensure it is Rule::program

        let mut state_graph_program = StateGraphProgram {
            title: String::new(),
            version: String::new(),
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
            //rule:title_content+rule:number+rule:stmt_list
            [(Rule::title_content, p), (Rule::number, p1), (Rule::stmt_list, p2)] => {
                state_graph_program.title = p.as_str().to_owned();
                state_graph_program.version = p1
                    .to_owned()
                    .into_inner()
                    .next()
                    .unwrap()
                    .into_inner()
                    .next()
                    .unwrap()
                    .as_str()
                    .to_owned();
                state_graph_program.stmts = p2
                    .to_owned()
                    .into_inner()
                    .map(|pair| Stmt::from_pair(&pair, config))
                    .collect::<Result<Vec<Stmt>, _>>()?;
            }
            //rule:title_content+rule:stmt_list
            [(Rule::title_content, p), (Rule::stmt_list, p1)] => {
                state_graph_program.title = p.as_str().to_owned();
                state_graph_program.stmts = p1
                    .to_owned()
                    .into_inner()
                    .map(|pair| Stmt::from_pair(&pair,config))
                    .collect::<Result<Vec<Stmt>, _>>()?;
            }

            //rule:number+rule:stmt_list
            [(Rule::number, p), (Rule::stmt_list, p1)] => {
                state_graph_program.version = p
                    .to_owned()
                    .into_inner()
                    .next()
                    .unwrap()
                    .into_inner()
                    .next()
                    .unwrap()
                    .as_str()
                    .to_owned();
                state_graph_program.stmts = p1
                    .to_owned()
                    .into_inner()
                    .map(|pair| Stmt::from_pair(&pair,config))
                    .collect::<Result<Vec<Stmt>, _>>()?;
            }
            //rule:stmt_list
            [(Rule::stmt_list, p)] => {
                state_graph_program.stmts = p
                    .to_owned()
                    .into_inner()
                    .map(|pair| Stmt::from_pair(&pair,config))
                    .collect::<Result<Vec<Stmt>, _>>()?;
            }

            _ => return Err(MermaidError::ParsingConversionError),
        }

        Ok(state_graph_program)
    }

    /// get outgoing neighbour state names and descriptions for a given state name
    /// the result set contains tuples of (neighbour_state_name, description), 
    /// the neighbour is reached via a state sequence statement starting from the given state name
    pub fn get_outgoing_neighbour_name_description(&self, state_name: &str) -> Vec<(String, String)> {
        let mut neighbours = vec![];

        for stmt in &self.stmts {
            match stmt {
                Stmt::StateSequenceStatement(seq_stmt) => {
                    if seq_stmt.get_start_state_name() == state_name {
                        let end_name = seq_stmt.get_end_state_name();
                        let description = seq_stmt.get_description();
                        neighbours.push((end_name, description));
                    }
                }
                _ => {}
            }
        }

        neighbours
    }

    /// get incoming neighbour state names and descriptions for a given state name
    /// the result set contains tuples of (neighbour_state_name, description),
    /// the neighbour is reached via a state sequence statement ending at the given state name
    pub fn get_incoming_neighbour_name_description(&self, state_name: &str) -> Vec<(String, String)> {
        let mut neighbours = vec![];

        for stmt in &self.stmts {
            match stmt {
                Stmt::StateSequenceStatement(seq_stmt) => {
                    if seq_stmt.get_end_state_name() == state_name {
                        let start_name = seq_stmt.get_start_state_name();
                        let description = seq_stmt.get_description();
                        neighbours.push((start_name, description));
                    }
                }
                _ => {}
            }
        }

        neighbours
    }

    /// get all type conversion triples (from_type, to_type, conversion_function)
    pub fn get_all_type_conversion_triples(&self) -> Vec<(String, String, String)> {
        let mut triples = vec![];

        for stmt in &self.stmts {
            match stmt {
                Stmt::StateSequenceStatement(seq_stmt) => {
                    let from_type = seq_stmt.get_start_state_name();
                    let to_type = seq_stmt.get_end_state_name();
                    let conversion_function = seq_stmt.get_description();
                    triples.push((from_type, to_type, conversion_function));
                }
                _ => {}
            }
        }

        triples
    }
}

///parse the rule:statement
#[derive(Debug, Clone, PartialEq, GenIsEnumVariant, EnumAccessors)]
pub enum Stmt {
    DirectStatement(DirectStatement),
    StateAsStatement(StateAsStatement),
    StateDescriptionStatement(StateDescriptionStatement),
    StateSequenceStatement(StateSequenceStatement),
    StateCompositeStatement(StateCompositeStatement),
    ForkStatement(ForkStatement),
    ChoiceStatement(ChoiceStatement),
    NoteStateStatement(NoteStateStatement),
    ConcurrencyStatement,
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
            [(Rule::direction, p)] => Ok(Stmt::DirectStatement(DirectStatement::from_pair(&p,config)?)),

            [(Rule::state_aliases_type, p)] => {
                Ok(Stmt::StateAsStatement(StateAsStatement::from_pair(&p,config)?))
            }

            [(Rule::state_description_type, p)] => Ok(Stmt::StateDescriptionStatement(
                StateDescriptionStatement::from_pair(&p,config)?,
            )),

            [(Rule::state_start_to_end, p)] => Ok(Stmt::StateSequenceStatement(
                StateSequenceStatement::from_pair(&p,config)?,
            )),

            [(Rule::state_composite, p)] => Ok(Stmt::StateCompositeStatement(
                StateCompositeStatement::from_pair(&p,config)?,
            )),

            [(Rule::forks_statement, p)] => Ok(Stmt::ForkStatement(ForkStatement::from_pair(&p,config)?)),

            [(Rule::choice_statement, p)] => {
                Ok(Stmt::ChoiceStatement(ChoiceStatement::from_pair(&p,config)?))
            }

            [(Rule::note_state_statement, p)] => {
                Ok(Stmt::NoteStateStatement(NoteStateStatement::from_pair(&p,config)?))
            }

            [(Rule::concurrency_statement, _)] => Ok(Stmt::ConcurrencyStatement),

            _ => Err(MermaidError::ParsingConversionError),
        }
    }
}

///parse the rule:commit_statement
pub type Direction = String;
#[derive(Debug, Clone, PartialEq)]
pub struct DirectStatement(Direction);
impl DirectStatement {
    pub fn from_pair(pair: &Pair<Rule>, _config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::direction); //ensure it is Rule::direction

        let list = pair
            .to_owned()
            .into_inner()
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();

        let v = list.as_slice();

        match v {
            [(Rule::direction_orientation, pair)] => Ok(DirectStatement(pair.as_str().to_owned())),
            _ => Err(MermaidError::ParsingConversionError),
        }
    }
}

///parse the rule:cherry_pick_statement
#[derive(Debug, Clone, PartialEq)]
pub struct StateAsStatement {
    state_description: String,
    state_alias: String,
}
impl StateAsStatement {
    pub fn from_pair(pair: &Pair<Rule>, _config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::state_aliases_type); //ensure it is Rule::state_aliases_type

        let list = pair
            .to_owned()
            .into_inner()
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();

        let v = list.as_slice();

        match v {
            [(Rule::term, p), (Rule::term, p1)] => Ok(StateAsStatement {
                state_description: p
                    .to_owned()
                    .into_inner()
                    .next()
                    .unwrap()
                    .as_str()
                    .to_owned(),
                state_alias: p1
                    .to_owned()
                    .into_inner()
                    .next()
                    .unwrap()
                    .as_str()
                    .to_owned(),
            }),

            _ => Err(MermaidError::ParsingConversionError),
        }
    }
}

///parse the rule:state_description_type
#[derive(Debug, Clone, PartialEq)]
pub struct StateDescriptionStatement {
    state_name: String,
    state_description: String,
}
impl StateDescriptionStatement {
    pub fn from_pair(pair: &Pair<Rule>, _config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::state_description_type); //ensure it is Rule::state_description_type

        let list = pair
            .to_owned()
            .into_inner()
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();

        let v = list.as_slice();

        match v {
            [(Rule::term, p), (Rule::state_description_type_desc, p1)] => {
                Ok(StateDescriptionStatement {
                    state_description: p
                        .to_owned()
                        .into_inner()
                        .next()
                        .unwrap()
                        .as_str()
                        .to_owned(),
                    state_name: p1.as_str().to_owned(),
                })
            }

            _ => Err(MermaidError::ParsingConversionError),
        }
    }
}

///parse the rule:state_description_type
#[derive(Debug, Clone, PartialEq, Accessors)]
pub struct StateSequenceStatement {
    state_sequence_start: String,
    state_sequence_end: String,
    state_description: String,
}

impl StateSequenceStatement {
    pub fn from_pair(pair: &Pair<Rule>, _config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::state_start_to_end); //ensure it is Rule::state_start_to_end

        let list = pair
            .to_owned()
            .into_inner()
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();

        let v = list.as_slice();

        match v {
            [(Rule::state_start_to_end_id, p), (Rule::state_start_to_end_id, p1), (Rule::state_start_to_end_desc, p2)] => {
                Ok(StateSequenceStatement {
                    state_sequence_start: strip_quotes(p.as_str()).to_owned(),
                    state_sequence_end: strip_quotes(p1.as_str()).to_owned(),
                    state_description: strip_quotes(p2.as_str()).to_owned(),
                })
            }

            [(Rule::state_start_to_end_id, p), (Rule::state_start_to_end_id, p1)] => {
                Ok(StateSequenceStatement {
                    state_sequence_start: strip_quotes(p.as_str()).to_owned(),
                    state_sequence_end: strip_quotes(p1.as_str()).to_owned(),
                    state_description: String::new(),
                })
            }

            _ => Err(MermaidError::ParsingConversionError),
        }
    }

    /// get state names. if state name is *, it'll be replaced with "start" or "end"
    pub fn get_state_names(&self) -> Vec<String> {
        let mut names = vec![];
        if self.state_sequence_start == "[*]" {
            names.push("start".to_owned());
        } else {
            names.push(self.state_sequence_start.clone());
        }

        if self.state_sequence_end == "[*]" {
            names.push("end".to_owned());
        } else {
            names.push(self.state_sequence_end.clone());
        }

        names
    }

    pub fn get_start_state_name(&self) -> String {
        if self.state_sequence_start == "[*]" {
            "start".to_owned()
        } else {
            self.state_sequence_start.clone()
        }
    }

    pub fn get_end_state_name(&self) -> String {
        if self.state_sequence_end == "[*]" {
            "end".to_owned()
        } else {
            self.state_sequence_end.clone()
        }
    }

    /// get description. if description is empty, it will be replace by start and end state name with underscore linked.
    pub fn get_description(&self) -> String {
        if self.state_description.is_empty() {
            format!(
                "{}_to_{}",
                self.get_start_state_name(), self.get_end_state_name()
            )
        } else {
            self.state_description.clone()
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StateCompositeStatement {
    state_name: String,
    stmts: Vec<Stmt>,
}
impl StateCompositeStatement {
    pub fn from_pair(pair: &Pair<Rule>,config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::state_composite); //ensure it is Rule::state_composite

        let list = pair
            .to_owned()
            .into_inner()
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();

        let v = list.as_slice();

        match v {
            [(Rule::term, p), (Rule::stmt_list, p1)] => Ok(StateCompositeStatement {
                state_name: p
                    .to_owned()
                    .into_inner()
                    .next()
                    .unwrap()
                    .as_str()
                    .to_owned(),

                stmts: p1
                    .to_owned()
                    .into_inner()
                    .map(|pair| Stmt::from_pair(&pair,config))
                    .collect::<Result<Vec<Stmt>, _>>()?,
            }),

            _ => Err(MermaidError::ParsingConversionError),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NoteStateStatement {
    SingeLine(NoteStateSingleLine),
    Block(NoteStateBlock),
}
impl NoteStateStatement {
    pub fn from_pair(pair: &Pair<Rule>,config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::note_state_statement); //ensure it is Rule::note_state_statement

        let list = pair
            .to_owned()
            .into_inner()
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();

        let v = list.as_slice();

        match v {
            [(Rule::note_state_singline_statement, p)] => Ok(NoteStateStatement::SingeLine(
                NoteStateSingleLine::from_pair(&p,config)?,
            )),

            [(Rule::note_state_block_statement, p)] => {
                Ok(NoteStateStatement::Block(NoteStateBlock::from_pair(&p,config)?))
            }

            _ => Err(MermaidError::ParsingConversionError),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct NoteStateSingleLine {
    orientation: Orientation,
    state_name: String,
    state_description: String,
}
impl NoteStateSingleLine {
    pub fn from_pair(pair: &Pair<Rule>, _config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::note_state_singline_statement); //ensure it is Rule::note_state_singline_statement

        let list = pair
            .to_owned()
            .into_inner()
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();

        let v = list.as_slice();

        match v {
            [(Rule::note_state_orientation, p), (Rule::term, p1), (Rule::note_state_singline_desc, p2)] =>
            {
                Ok(NoteStateSingleLine {
                    orientation: p.as_str().to_owned(),

                    state_name: p1
                        .to_owned()
                        //fetch terminal rule
                        .into_inner()
                        .next()
                        .unwrap()
                        .as_str()
                        .to_owned(),
                    state_description: p2.as_str().to_owned(),
                })
            }

            _ => Err(MermaidError::ParsingConversionError),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NoteStateBlock {
    orientation: Orientation,
    state_name: String,
    state_description: String,
}
impl NoteStateBlock {
    pub fn from_pair(pair: &Pair<Rule>, _config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::note_state_block_statement); //ensure it is Rule::note_state_block_statement

        let list = pair
            .to_owned()
            .into_inner()
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();

        let v = list.as_slice();

        match v {
            [(Rule::note_state_orientation, p), (Rule::note_state_block_id, p1), (Rule::note_state_block_desc, p2)] => {
                Ok(NoteStateBlock {
                    orientation: p.as_str().to_owned(),
                    state_name: p1.as_str().to_owned(),
                    state_description: p2.as_str().to_owned(),
                })
            }

            _ => Err(MermaidError::ParsingConversionError),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct ForkStatement(String);

impl ForkStatement {
    pub fn from_pair(pair: &Pair<Rule>, _config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::forks_statement); //ensure it is Rule::forks_statement

        let list = pair
            .to_owned()
            .into_inner()
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();

        let v = list.as_slice();

        match v {
            [(Rule::term, p)] => Ok(ForkStatement(
                p.to_owned()
                    .into_inner()
                    .next()
                    .unwrap()
                    .into_inner()
                    .next()
                    .unwrap()
                    .as_str()
                    .to_owned(),
            )),

            _ => Err(MermaidError::ParsingConversionError),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChoiceStatement(String);

impl ChoiceStatement {
    pub fn from_pair(pair: &Pair<Rule>, _config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::choice_statement); //ensure it is Rule::choice_statement

        let list = pair
            .to_owned()
            .into_inner()
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();

        let v = list.as_slice();

        match v {
            [(Rule::term, p)] => Ok(ChoiceStatement(
                p.to_owned()
                    .into_inner()
                    .next()
                    .unwrap()
                    .into_inner()
                    .next()
                    .unwrap()
                    .as_str()
                    .to_owned(),
            )),

            _ => Err(MermaidError::ParsingConversionError),
        }
    }
}