use std::collections::HashMap;

use crate::mermaid_error::*;

use super::CodeGenConfiguration;
use pest::iterators::Pair;
use pest_derive::Parser;
use rust_macro::*;

#[derive(Parser)]
#[grammar = "basic_type_grammar.pest"]
#[grammar = "mermaid_sequence/sequence.pest"]
pub(crate) struct SequenceParser;

pub type TimeLineTitle = String;

#[derive(Debug, Clone)]
pub struct SequenceProgram {
    pub stmts: Vec<Stmt>,
}

impl SequenceProgram {
    pub fn from_pair(pair: &Pair<Rule>,config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::program); //ensure it is Rule::program

        let mut sequence_program = SequenceProgram { stmts: vec![] };

        sequence_program.stmts = pair
            .to_owned()
            .into_inner()
            .filter(|pair| pair.as_rule() == Rule::stmt_list)
            .flat_map(|pair| pair.into_inner())
            .map(|pair| Stmt::from_pair(&pair,config))
            .collect::<Result<Vec<Stmt>, _>>()?;

        Ok(sequence_program)
    }

    /// Get all participants/actors defined in the sequence diagram
    pub fn get_all_participants(&self) -> HashMap<String, String> {
        let mut participants = HashMap::new();

        for stmt in &self.stmts {
            match stmt {
                Stmt::ParticipantStatement(p) => {
                    participants.insert(p.actor_aliases.clone(), p.actor_name.clone());
                }
                Stmt::ActorStatement(a) => {
                    participants.insert(a.actor_aliases.clone(), a.actor_name.clone());
                }
                _ => {}
            }
        }

        participants
    }

    pub fn get_creation(&self) -> Vec<ActorCreationStatement> {
        let mut creations = vec![];

        for stmt in &self.stmts {
            match stmt {
                Stmt::ActorCreationStatement(ac) => {
                    creations.push(ac.clone());
                }
                _ => {}
            }
        }

        creations
    }

    pub fn get_destory(&self) -> Vec<DestroyStatement> {
        let mut destories = vec![];

        for stmt in &self.stmts {
            match stmt {
                Stmt::DestroyStatement(ds) => {
                    destories.push(ds.clone());
                }
                _ => {}
            }
        }

        destories
    }

    /// Get all participant/actor name and alias pairs
    pub fn get_participant_name_alias(&self) -> Vec<(String, String)> {
        let participant_aliases = self.get_all_participants();

        let mut participant_name_alias = vec![];
        for stmt in &self.stmts {
            match stmt {
                Stmt::MessageStatement(n) => {
                    let name = &n.message_actor_from;
                    if participant_aliases.contains_key(name) {
                        let alias = participant_aliases.get(name).unwrap().to_string();  // get original name which is called alias here
                        participant_name_alias.push((alias, name.to_string()));
                    }
                    else {
                        participant_name_alias.push((name.to_string(), name.to_string()));
                    }

                    let name = &n.message_actor_to;
                    if participant_aliases.contains_key(name) {
                        let alias = participant_aliases.get(name).unwrap().to_string(); // get original name which is called alias here
                        participant_name_alias.push((alias, name.to_string()));
                    }
                    else {
                        participant_name_alias.push((name.to_string(), name.to_string()));
                    }
                }
                _ => {}
            }
        }

        // remove names found in creation and destory statements
        let creations = self.get_creation();
        for creation in creations {
            let name = creation.get_actor_name();
            let alias = creation.get_actor_aliases();
            participant_name_alias.retain(|x| x.0 != name && x.1 != alias);
        }

        let destories = self.get_destory();
        for destory in destories {
            let name = destory.get_actor_name().to_string();
            participant_name_alias.retain(|x| x.0 != name);
        }

        // remove duplication
        participant_name_alias.sort();
        participant_name_alias.dedup();

        participant_name_alias
    }

    // get all messages whose from is defined as name or alias in the function parameters
    pub fn get_messages_by_name_alias(&self, name:&str, alias:&str) -> Vec<&MessageStatement> {
        let mut messages = vec![];

        for stmt in &self.stmts {
            match stmt {
                Stmt::MessageStatement(m) => {
                    if m.message_actor_from == name || m.message_actor_from == alias {
                        messages.push(m);
                    }
                }
                _ => {}
            }
        }

        messages
    }
}

#[derive(Debug, Clone, PartialEq, EnumAccessors, GenIsEnumVariant)]
pub enum Stmt {
    ActorStatement(ActorStatement),
    ParticipantStatement(ParticipantStatement),
    DestroyStatement(DestroyStatement),
    LoopsStatement(LoopsStatement),
    AltStatement(AltStatement),
    OptStatement(OptStatement),
    ParallelStatement(ParallelStatement),
    CriticalStatement(CriticalStatement),
    BreakStatement(BreakStatement),
    BackgroundHighlightStatement(BackgroundHighlightStatement),
    MessageStatement(MessageStatement),
    ActivationStatement(ActivationStatement),
    NotesStatement(NotesStatement),
    BoxStatement(BoxStatement),
    ActorCreationStatement(ActorCreationStatement),
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
            [(Rule::actor_statement, p)] => Ok(Stmt::ActorStatement(ActorStatement::from_pair(p,config)?)),

            [(Rule::participant_statement, p)] => Ok(Stmt::ParticipantStatement(
                ParticipantStatement::from_pair(&p,config)?,
            )),

            [(Rule::actor_destroy_statement, p)] => {
                Ok(Stmt::DestroyStatement(DestroyStatement::from_pair(&p,config)?))
            }

            [(Rule::loops_statement, p)] => {
                Ok(Stmt::LoopsStatement(LoopsStatement::from_pair(&p,config)?))
            }

            [(Rule::alt_statement, p)] => Ok(Stmt::AltStatement(AltStatement::from_pair(&p,config)?)),

            [(Rule::opt_statement, p)] => Ok(Stmt::OptStatement(OptStatement::from_pair(&p,config)?)),

            [(Rule::parallel_statement, p)] => {
                Ok(Stmt::ParallelStatement(ParallelStatement::from_pair(&p,config)?))
            }

            [(Rule::critical_statement, p)] => {
                Ok(Stmt::CriticalStatement(CriticalStatement::from_pair(&p,config)?))
            }

            [(Rule::break_statement, p)] => {
                Ok(Stmt::BreakStatement(BreakStatement::from_pair(&p,config)?))
            }

            [(Rule::background_highlight_statement, p)] => Ok(Stmt::BackgroundHighlightStatement(
                BackgroundHighlightStatement::from_pair(&p,config)?,
            )),

            [(Rule::message_statement, p)] => {
                Ok(Stmt::MessageStatement(MessageStatement::from_pair(&p,config)?))
            }

            [(Rule::activation_statement, p)] => Ok(Stmt::ActivationStatement(
                ActivationStatement::from_pair(&p,config)?,
            )),

            [(Rule::notes_statement, p)] => {
                Ok(Stmt::NotesStatement(NotesStatement::from_pair(&p,config)?))
            }

            [(Rule::box_statement, p)] => Ok(Stmt::BoxStatement(BoxStatement::from_pair(&p,config)?)),

            [(Rule::actor_creation_statement, p)] => Ok(Stmt::ActorCreationStatement(
                ActorCreationStatement::from_pair(&p,config)?,
            )),

            _ => Err(MermaidError::ParsingConversionError),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActorStatement {
    actor_name: String,
    actor_aliases: String,
}
impl ActorStatement {
    pub fn from_pair(pair: &Pair<Rule>, _config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::actor_statement); //ensure it is Rule::actor_statement

        let list = pair
            .to_owned()
            .into_inner()
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();

        let v = list.as_slice();

        match v {
            [(Rule::actor_role, p), (Rule::actor_role, p1)] => {
                let actor_name = p.as_str().to_owned();

                let actor_aliases = p1.as_str().to_owned();

                Ok(ActorStatement {
                    actor_name,
                    actor_aliases,
                })
            }

            [(Rule::actor_role, p)] => {
                let actor_name = p.as_str().to_owned();

                let actor_aliases = String::new();

                Ok(ActorStatement {
                    actor_name,
                    actor_aliases,
                })
            }

            _ => Err(MermaidError::ParsingConversionError),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticipantStatement {
    actor_name: String,
    actor_aliases: String,
}
impl ParticipantStatement {
    pub fn from_pair(pair: &Pair<Rule>, _config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::participant_statement); //ensure it is Rule::participant_statement

        let list = pair
            .to_owned()
            .into_inner()
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();

        let v = list.as_slice();

        match v {
            [(Rule::participant_role, p), (Rule::participant_role, p1)] => {
                let actor_name = p.as_str().to_owned();

                let actor_aliases = p1.as_str().to_owned();

                Ok(ParticipantStatement {
                    actor_name,
                    actor_aliases,
                })
            }

            [(Rule::participant_role, p)] => {
                let actor_name = p.as_str().to_owned();

                let actor_aliases = String::new();

                Ok(ParticipantStatement {
                    actor_name,
                    actor_aliases,
                })
            }

            _ => Err(MermaidError::ParsingConversionError),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Accessors)]
pub struct DestroyStatement {
    actor_name: String,
}

impl DestroyStatement {
    pub fn from_pair(pair: &Pair<Rule>, _config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::actor_destroy_statement); //ensure it is Rule::actor_destroy_statement

        let list = pair
            .to_owned()
            .into_inner()
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();

        let v = list.as_slice();

        match v {
            [(Rule::actor_destroy_role, p)] => {
                let actor_name = p.as_str().to_owned();

                Ok(DestroyStatement { actor_name })
            }

            _ => Err(MermaidError::ParsingConversionError),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoopsStatement {
    loop_condition: String,
    stmts: Vec<Stmt>,
}
impl LoopsStatement {
    pub fn from_pair(pair: &Pair<Rule>,config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::loops_statement); //ensure it is Rule::loops_statement

        let list = pair
            .to_owned()
            .into_inner()
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();

        let v = list.as_slice();

        match v {
            [(Rule::loops_statement_text, p), (Rule::stmt_list, p1)] => {
                let loop_condition = p.as_str().to_owned();

                let stmts = p1
                    .to_owned()
                    .into_inner()
                    .map(|pair| Stmt::from_pair(&pair,config))
                    .collect::<Result<Vec<Stmt>, _>>()?;

                Ok(LoopsStatement {
                    loop_condition,
                    stmts,
                })
            }

            _ => Err(MermaidError::ParsingConversionError),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AltStatement {
    alt_text: String,
    alt_stmts: Vec<Stmt>,
    alt_else_text: String,
    alt_else_stmts: Vec<Stmt>,
}
impl AltStatement {
    pub fn from_pair(pair: &Pair<Rule>,config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::alt_statement); //ensure it is Rule::alt_statement

        let list = pair
            .to_owned()
            .into_inner()
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();

        let v = list.as_slice();

        match v {
            [(Rule::alt_statement_text, p), (Rule::stmt_list, p1), (Rule::alt_statement_text, p2), (Rule::stmt_list, p3)] =>
            {
                let alt_text = p.as_str().to_owned();

                let alt_stmts = p1
                    .to_owned()
                    .into_inner()
                    .map(|pair| Stmt::from_pair(&pair,config))
                    .collect::<Result<Vec<Stmt>, _>>()?;

                let alt_else_text = p2.as_str().to_owned();

                let alt_else_stmts = p3
                    .to_owned()
                    .into_inner()
                    .map(|pair| Stmt::from_pair(&pair,config))
                    .collect::<Result<Vec<Stmt>, _>>()?;

                Ok(AltStatement {
                    alt_text,
                    alt_stmts,
                    alt_else_text,
                    alt_else_stmts,
                })
            }

            _ => Err(MermaidError::ParsingConversionError),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BreakStatement {
    break_statement_action: String,
    break_statement_stmts: Vec<Stmt>,
}
impl BreakStatement {
    pub fn from_pair(pair: &Pair<Rule>,config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::break_statement); //ensure it is Rule::break_statement

        let list = pair
            .to_owned()
            .into_inner()
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();

        let v = list.as_slice();

        match v {
            [(Rule::break_statement_action, p), (Rule::stmt_list, p1)] => {
                let break_statement_action = p.as_str().to_owned();

                let break_statement_stmts = p1
                    .to_owned()
                    .into_inner()
                    .map(|pair| Stmt::from_pair(&pair,config))
                    .collect::<Result<Vec<Stmt>, _>>()?;

                Ok(BreakStatement {
                    break_statement_action,
                    break_statement_stmts,
                })
            }

            _ => Err(MermaidError::ParsingConversionError),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BackgroundHighlightStatement {
    background_object: String,
    background_colors: String,
    background_stmts: Vec<Stmt>,
}
impl BackgroundHighlightStatement {
    pub fn from_pair(pair: &Pair<Rule>,config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::background_highlight_statement); //ensure it is Rule::background_highlight_statement

        let list = pair
            .to_owned()
            .into_inner()
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();

        let v = list.as_slice();

        match v {
            [(Rule::term, p), (Rule::background_highlight_colors, p1), (Rule::stmt_list, p2)] => {
                let background_object = p
                    .to_owned()
                    .into_inner()
                    .next()
                    .unwrap()
                    .as_str()
                    .to_owned();

                let background_colors = p1
                    .to_owned()
                    .into_inner()
                    .map(|pair| {
                        pair.into_inner()
                            .next()
                            .unwrap()
                            .into_inner()
                            .next()
                            .unwrap()
                            .as_str()
                            .to_owned()
                    })
                    .collect::<Vec<_>>()
                    .join(",");

                let background_stmts = p2
                    .to_owned()
                    .into_inner()
                    .map(|pair| Stmt::from_pair(&pair,config))
                    .collect::<Result<Vec<Stmt>, _>>()?;

                Ok(BackgroundHighlightStatement {
                    background_object,
                    background_colors,
                    background_stmts,
                })
            }

            _ => Err(MermaidError::ParsingConversionError),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Accessors)]
pub struct MessageStatement {
    message_actor_from: String,
    message_actor_to: String,
    message_text: String,
}

impl MessageStatement {
    pub fn from_pair(pair: &Pair<Rule>, _config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::message_statement); //ensure it is Rule::message_statement

        let list = pair
            .to_owned()
            .into_inner()
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();

        let v = list.as_slice();

        match v {
            [(Rule::message_actor, p), (Rule::message_actor, p1), (Rule::message_text, p2)] => {
                let message_actor_from = p.as_str().to_owned();

                let message_actor_to = p1.as_str().to_owned();

                let message_text = p2.as_str().to_owned();

                Ok(MessageStatement {
                    message_actor_from,
                    message_actor_to,
                    message_text,
                })
            }

            _ => Err(MermaidError::ParsingConversionError),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActivationStatement {
    activation_role: String,
    activation_action: String,
}
impl ActivationStatement {
    pub fn from_pair(pair: &Pair<Rule>, _config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::activation_statement); //ensure it is Rule::activation_statement

        let list = pair
            .to_owned()
            .into_inner()
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();

        let v = list.as_slice();

        match v {
            [(Rule::activation_action, p), (Rule::activation_role, p1)] => {
                let activation_action = p.as_str().to_owned();

                let activation_role = p1.as_str().to_owned();

                Ok(ActivationStatement {
                    activation_action,
                    activation_role,
                })
            }

            _ => Err(MermaidError::ParsingConversionError),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NotesStatement {
    notes_statement_direction: String,
    notes_statement_actor: String,
    notes_statement_text: String,
}
impl NotesStatement {
    pub fn from_pair(pair: &Pair<Rule>, _config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::notes_statement); //ensure it is Rule::notes_statement

        let list = pair
            .to_owned()
            .into_inner()
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();

        let v = list.as_slice();

        match v {
            [(Rule::notes_statement_direction, p), (Rule::notes_actor, p1), (Rule::notes_statement_message, p2)] =>
            {
                let notes_statement_direction = p.as_str().to_owned();

                let notes_statement_actor = p1.as_str().to_owned();

                let notes_statement_text = p2.as_str().to_owned();

                Ok(NotesStatement {
                    notes_statement_direction,
                    notes_statement_actor,
                    notes_statement_text,
                })
            }

            _ => Err(MermaidError::ParsingConversionError),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoxStatement {
    box_statement_text: String,
    box_statement_stmts: Vec<Stmt>,
}
impl BoxStatement {
    pub fn from_pair(pair: &Pair<Rule>,config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::box_statement); //ensure it is Rule::box_statement

        let list = pair
            .to_owned()
            .into_inner()
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();

        let v = list.as_slice();

        match v {
            [(Rule::box_statement_text, p), (Rule::stmt_list, p1)] => {
                let box_statement_text = p.as_str().to_owned();

                let box_statement_stmts = p1
                    .to_owned()
                    .into_inner()
                    .map(|pair| Stmt::from_pair(&pair,config))
                    .collect::<Result<Vec<Stmt>, _>>()?;

                Ok(BoxStatement {
                    box_statement_text,
                    box_statement_stmts,
                })
            }

            _ => Err(MermaidError::ParsingConversionError),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActorCreationStatement {
    Participant(ParticipantStatement),
    Actor(ActorStatement),
}

impl ActorCreationStatement {
    pub fn from_pair(pair: &Pair<Rule>,config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::actor_creation_statement); //ensure it is Rule::actor_creation_statement

        let list = pair
            .to_owned()
            .into_inner()
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();

        let v = list.as_slice();

        match v {
            [(Rule::participant_statement, p)] => Ok(ActorCreationStatement::Participant(
                ParticipantStatement::from_pair(&p,config)?,
            )),

            [(Rule::actor_statement, p)] => Ok(ActorCreationStatement::Actor(
                ActorStatement::from_pair(&p,config)?,
            )),

            _ => Err(MermaidError::ParsingConversionError),
        }
    }

    pub fn get_actor_name(&self) -> String {
        match self {
            ActorCreationStatement::Participant(p) => p.actor_name.clone(),
            ActorCreationStatement::Actor(a) => a.actor_name.clone(),
        }
    }

    pub fn get_actor_aliases(&self) -> String {
        match self {
            ActorCreationStatement::Participant(p) => p.actor_aliases.clone(),
            ActorCreationStatement::Actor(a) => a.actor_aliases.clone(),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct OptStatement {
    opt_text: String,
    opt_stmts: Vec<Stmt>,
}
impl OptStatement {
    pub fn from_pair(pair: &Pair<Rule>,config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::opt_statement); //ensure it is Rule::opt_statement

        let list = pair
            .to_owned()
            .into_inner()
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();

        let v = list.as_slice();

        match v {
            [(Rule::opt_statement_text, p), (Rule::stmt_list, p1)] => {
                let opt_text = p.as_str().to_owned();

                let opt_stmts = p1
                    .to_owned()
                    .into_inner()
                    .map(|pair| Stmt::from_pair(&pair,config))
                    .collect::<Result<Vec<Stmt>, _>>()?;

                Ok(OptStatement {
                    opt_text,
                    opt_stmts,
                })
            }

            _ => Err(MermaidError::ParsingConversionError),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParallelStatement {
    parallel_actions: Vec<(String, Vec<Stmt>)>,
}
impl ParallelStatement {
    pub fn from_pair(pair: &Pair<Rule>,config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::parallel_statement); //ensure it is Rule::parallel_statement

        let mut parallel_actions = vec![];

        let list = pair
            .to_owned()
            .into_inner()
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();

        let v = list.as_slice();

        match v {
            [(Rule::parallel_statement_action, p), (Rule::stmt_list, p1), (Rule::parallel_action_statements, p2)] =>
            {
                //parse the parallel_action_text in the section "par"
                let parallel_action_text = p.as_str().to_owned();

                //parse the parallel_action_statements in the section "par"
                let parallel_action_statements = p1
                    .to_owned()
                    .into_inner()
                    .map(|pair| Stmt::from_pair(&pair,config))
                    .collect::<Result<Vec<Stmt>, _>>()?;

                //add the parsed parallel_action_text and parallel_action_statements
                // in the section "par" to vector
                parallel_actions.push((parallel_action_text, parallel_action_statements));

                //parse the parallel_action_text and parallel_action_statements
                //in the section "and"
                p2.to_owned()
                    .into_inner()
                    .collect::<Vec<Pair<Rule>>>()
                    //since parallel_action_text and parallel_action_statements
                    //are paired for each section "and"
                    .chunks(2)
                    .map(|slice| {
                        let parallel_action_text = slice[0].as_str().to_owned();
                        let parallel_action_statements = slice[1]
                            .to_owned()
                            .into_inner()
                            .map(|pair| Stmt::from_pair(&pair,config))
                            .collect::<Result<Vec<Stmt>, _>>()
                            .unwrap();
                        (parallel_action_text, parallel_action_statements)
                    })
                    .for_each(|pair| {
                        parallel_actions.push(pair);
                    });

                Ok(ParallelStatement { parallel_actions })
            }

            _ => Err(MermaidError::ParsingConversionError),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CriticalStatement {
    critical_actions: Vec<(String, Vec<Stmt>)>,
}
impl CriticalStatement {
    pub fn from_pair(pair: &Pair<Rule>,config:&mut CodeGenConfiguration) -> Result<Self, MermaidError> {
        assert_eq!(pair.as_rule(), Rule::critical_statement); //ensure it is Rule::critical_statement

        let mut critical_actions = vec![];

        let list = pair
            .to_owned()
            .into_inner()
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();

        let v = list.as_slice();

        match v {
            [(Rule::critical_statement_action, p), (Rule::stmt_list, p1), (Rule::critical_action_statements, p2)] =>
            {
                //parse the critical_action_text in the section "critical"
                let critical_action_text = p.as_str().to_owned();

                //parse the critical_action_statements in the section "critical"
                let critical_action_statements = p1
                    .to_owned()
                    .into_inner()
                    .map(|pair| Stmt::from_pair(&pair,config))
                    .collect::<Result<Vec<Stmt>, _>>()?;

                //add the parsed critical_action_text and critical_action_statements
                // in the section "critical" to vector
                critical_actions.push((critical_action_text, critical_action_statements));

                //parse the critical_action_text and critical_action_statements
                //in the section "option"
                p2.to_owned()
                    .into_inner()
                    .collect::<Vec<Pair<Rule>>>()
                    //since critical_action_text and critical_action_statements
                    //are paired for each section "option"
                    .chunks(2)
                    .map(|slice| {
                        let critical_action_text = slice[0].as_str().to_owned();
                        let critical_action_statements = slice[1]
                            .to_owned()
                            .into_inner()
                            .map(|pair| Stmt::from_pair(&pair,config))
                            .collect::<Result<Vec<Stmt>, _>>()
                            .unwrap();
                        (critical_action_text, critical_action_statements)
                    })
                    .for_each(|pair| {
                        critical_actions.push(pair);
                    });

                Ok(CriticalStatement { critical_actions })
            }

            _ => Err(MermaidError::ParsingConversionError),
        }
    }
}