use std::collections::HashMap;
use core_utils::debug::*;
use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

use crate::pest_parser::pest_expression::PestExpression;

use super::{code_gen_config::CodeGenConfiguration, pest_lang_err::PestLangError, pest_rule_list::PestRuleList, traits::to_pest_text::ToPestText};

#[derive(Parser)]
#[grammar = "basic_type_grammar.pest"]
#[grammar = "pest_parser/pest.pest"]
pub struct PestParser;

pub fn pest_parse(str:&str) -> Result<Vec<PestRule>, PestLangError> {
    let pairs = PestParser::parse(Rule::prog, str).map_err(|err| PestLangError::General(format!("parsing error {err:?}")))?;
    let config = &mut CodeGenConfiguration::default();
    let mut rr = Vec::default();
    for p in pairs {
        if p.as_rule() == Rule::EOI {
            continue;
        }

        let r = PestRule::from_pair(&p, config)?;
        rr.push(r);
    }
    
    Ok(rr)
}

#[derive(Debug, Clone)]
pub struct PestRule {
    name : String,
    expression : PestExpression,
    tag : Option<PestRuleTag>,

    // rule properties
    can_generate_type : bool,
}

impl PestRule {
    pub fn from_pair(pair:&Pair<Rule>, config:&mut CodeGenConfiguration) -> Result<Self, PestLangError> { 
        assert!(pair.as_rule() == Rule::rule);

        let inner = pair.to_owned().into_inner();
        let pairs = inner.map(|x| (x.as_rule(), x)).collect::<Vec<_>>();
        match pairs.as_slice() {
            [(Rule::identifier, p), (Rule::rule_tag, p1), (Rule::pest_expression, p2)] => {
                let name = p.as_str().to_string();
                let tag = Some(PestRuleTag::from_pair(p1, config)?);
                let expression = PestExpression::from_pair(p2, config)?;
                Ok(Self { name, expression, tag, can_generate_type : false })
            }
            [(Rule::identifier, p), (Rule::pest_expression, p2)] => {
                let name = p.as_str().to_string();
                let expression = PestExpression::from_pair(p2, config)?;
                Ok(Self { name, expression, tag : None, can_generate_type : false })
            }
            exs => {
                error_string(format!("missing following rule sequence"));
                for ex in exs {
                    error_string(format!("{:?}", ex.1));
                } 
                Err(PestLangError::MissingCase(format!("PestRule: missed case {pairs:?}")))
            }
        }
    }

    pub fn get_name(&self) -> String {
        self.name.to_string()
    }

    pub fn get_expression(&self) -> &PestExpression {
        &self.expression
    }

    pub fn get_expression_mut(&mut self) -> &mut PestExpression {
        &mut self.expression
    }

    pub fn new(str:&str, expression:PestExpression, tag:Option<PestRuleTag>) -> Self {
        Self { name : str.to_string(), expression, tag, can_generate_type : false }
    }

    pub fn get_can_generate_type(&self) -> bool {
        self.can_generate_type
    }

    pub fn set_can_generate_type(&mut self, value:bool) {
        self.can_generate_type = value;
        self.expression.set_can_generate_type(value);
    }

    pub fn compute_can_generate_type(&self, rules:&PestRuleList) -> bool {
        let can_generate = self.expression.compute_can_generate_type(rules);
        if !can_generate && self.is_hidden() {
            false
        }
        else {
            true
        }
    }

    /// set expression's tag as rule's name
    pub fn set_expression_tag(&mut self) {
        self.expression.set_tag(self.get_name().as_str());
        self.expression.set_sub_expression_tag();
    }

    pub fn is_hidden(&self) -> bool {
        if let Some(tag) = &self.tag {
            if tag == &PestRuleTag::Hidden {
                return true;
            }
        }
        false
    }

    pub fn finalize_can_generate_type(&mut self, mapping:&HashMap<String, bool>) {
        self.expression.finalize_can_generate_type(mapping);
        self.can_generate_type = self.expression.get_can_generate_type();
    }
}

impl ToPestText for PestRule {
    fn to_pest_text(&self) -> String {
        let tag_text = if let Some(tag) = &self.tag {
            tag.to_pest_text()
        } else {
            "".to_string()
        };
        format!("{} = {tag_text}{}", self.name, self.expression.to_pest_text())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PestRuleTag{
    Atom,
    Hidden,
    Bang, 
    Atom2, //$ sign
}

impl PestRuleTag {
    pub fn from_pair(pair:&Pair<Rule>, _config:&mut CodeGenConfiguration) -> Result<Self, PestLangError> { 
        let rr = pair.as_rule();
        assert!(rr == Rule::rule_tag);

        match pair.as_str() {
            "_" => Ok(Self::Hidden),
            "@" => Ok(Self::Atom),
            "!" => Ok(Self::Bang),
            "$" => Ok(Self::Atom2),
            _ => Err(PestLangError::MissingCase(format!("PestRuleTag: Missing case {pair:?}")))
        }
    }
}

impl ToPestText for PestRuleTag {
    fn to_pest_text(&self) -> String {
        match self {
            PestRuleTag::Atom => "@".to_string(),
            PestRuleTag::Hidden => "_".to_string(),
            PestRuleTag::Bang => "!".to_string(),
            PestRuleTag::Atom2 => "$".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PestFunction {
    function_name : String, 
    parameters : Vec<String>,
}

impl PestFunction {
    pub fn from_pair(pair:&Pair<Rule>, config:&mut CodeGenConfiguration) -> Result<Self, PestLangError> { 
        let inner = pair.to_owned().into_inner();
        let pairs = inner.map(|x| (x.as_rule(), x)).collect::<Vec<_>>();
        match pairs.as_slice() {
            [(Rule::pest_function_name, p)] => Ok(Self { function_name: p.as_str().to_string(), parameters: Vec::default() }),
            [(Rule::pest_function_name, p), (Rule::string, p2)] => 
                Ok(Self { function_name: p.as_str().to_string(), parameters: vec![p2.as_str().to_string()] }),
            [(Rule::pest_function_name, p), (Rule::identifier, p2)] => 
                Ok(Self { function_name: p.as_str().to_string(), parameters: vec![p2.as_str().to_string()] }),
            _ => Err(PestLangError::MissingCase(format!("in PestFunction: cannot process {pairs:?}"))),
        }
    }

    pub fn new(name:&str, params:Vec<String>) -> Self {
        Self { function_name : name.to_string(), parameters : params }
    }

}

impl ToPestText for PestFunction {
    fn to_pest_text(&self) -> String {
        let params = self.parameters.join(", ");

        if params.is_empty() {
            format!("{}()", self.function_name)
        } else {
            format!("{}({})", self.function_name, params)
        }
    }
}
