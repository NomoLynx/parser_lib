use pest::iterators::Pair;

use core_utils::debug::*;

use super::{code_gen_config::CodeGenConfiguration, pest_expression::PestExpression, pest_lang_err::PestLangError, pest_pest::*, pest_rule_list::PestRuleList, traits::to_pest_text::ToPestText};

#[derive(Debug, Clone)]
pub enum PestAtom {
    String(Option<PestPreOperator>, String, Option<PestPostOperator>),
    Id(Option<PestPreOperator>, String, Option<PestPostOperator>),
    Expression(Option<PestPreOperator>, Box<PestExpression>, Option<PestPostOperator>),
    Function(Option<PestPreOperator>, PestFunction, Option<Box<PestAtom>>, Option<PestPostOperator>),
    Range(String, String),
}

impl PestAtom {
    pub fn from_pair(pair:&Pair<Rule>, config:&mut CodeGenConfiguration) -> Result<Self, PestLangError> { 
        let inner = pair.to_owned().into_inner();
        let pairs = inner.map(|x| (x.as_rule(), x)).collect::<Vec<_>>();
        match pairs.as_slice() {
            [(Rule::string, p)] => Ok(Self::String(None, p.as_str().to_string(), None)),
            [(Rule::string, p), (Rule::pest_post_op, p1)] => 
                Ok(Self::String(None, p.as_str().to_string(), Some(PestPostOperator::from_pair(p1, config)?))),
            [(Rule::identifier, p)] => Ok(Self::Id(None, p.as_str().to_string(), None)),
            [(Rule::identifier, p), (Rule::pest_post_op, p1)] => 
                Ok(Self::Id(None, p.as_str().to_string(), Some(PestPostOperator::from_pair(p1, config)?))),
            [(Rule::left_bracket, _), (Rule::pest_expression, p), (Rule::right_bracket, _)] |
            [(Rule::pest_expression, p)] => 
                Ok(Self::Expression(None, Box::new(PestExpression::from_pair(p, config)?), None)),
            [(Rule::left_bracket, _), (Rule::pest_expression, p), (Rule::right_bracket, _), (Rule::pest_post_op, p1)] |
            [(Rule::pest_expression, p), (Rule::pest_post_op, p1)] => 
                Ok(Self::Expression(None, Box::new(PestExpression::from_pair(p, config)?), Some(PestPostOperator::from_pair(p1, config)?))),
            [(Rule::pest_function, p)] => Ok(Self::Function(None, PestFunction::from_pair(p, config)?, None, None)),
            [(Rule::pest_function, p), (Rule::pest_post_op, p1)] => 
                Ok(Self::Function(None, PestFunction::from_pair(p, config)?, None, Some(PestPostOperator::from_pair(p1, config)?))),
            [(Rule::pest_function, p), (Rule::identifier, p1)] => {
                let function = PestFunction::from_pair(p, config)?;
                Ok(Self::Function(None, function, Some(Box::new(PestAtom::from_pair(p1, config)?)) ,None))
            }
            [(Rule::pest_function_name, p), (Rule::identifier, p1)] => {
                let function = PestFunction::new(p.as_str(), Vec::default());
                Ok(Self::Function(None, function, Some(Box::new(Self::Id(None, p1.as_str().to_string(), None))), None))
            }
            [(Rule::pest_function, p), (Rule::identifier, p1), (Rule::pest_post_op, p2)] => {
                let function = PestFunction::from_pair(p, config)?;
                Ok(Self::Function(None, function, Some(Box::new(PestAtom::from_pair(p1, config)?)), Some(PestPostOperator::from_pair(p2, config)?)))
            }
            [(Rule::pest_function_name, p), (Rule::identifier, p1), (Rule::pest_post_op, p2)] => {
                let function = PestFunction::new(p.as_str(), Vec::default());
                Ok(Self::Function(None, function, Some(Box::new(Self::Id(None, p1.as_str().to_string(), None))), Some(PestPostOperator::from_pair(p2, config)?)))
            }
            [(Rule::pest_function, p), (Rule::string, p1)] => {
                let function = PestFunction::from_pair(p, config)?;
                Ok(Self::Function(None, function, Some(Box::new(Self::String(None, p1.as_str().to_string(), None))), None))
            }
            [(Rule::pest_function_name, p), (Rule::string, p1)] => {
                let function = PestFunction::new(p.as_str(), Vec::default());
                Ok(Self::Function(None, function, Some(Box::new(Self::String(None, p1.as_str().to_string(), None))), None))
            }
            [(Rule::pest_function, p), (Rule::string, p1), (Rule::pest_post_op, p2)] => {
                let function = PestFunction::from_pair(p, config)?;
                Ok(Self::Function(None, function, Some(Box::new(Self::String(None, p1.as_str().to_string(), None))), Some(PestPostOperator::from_pair(p2, config)?)))
            }
            [(Rule::pest_pre_op, p0), (Rule::string, p)] => Ok(Self::String(Some(PestPreOperator::from_pair(p0, config)?), p.as_str().to_string(), None)),
            [(Rule::pest_pre_op, p0), (Rule::string, p), (Rule::pest_post_op, p1)] => 
                Ok(Self::String(Some(PestPreOperator::from_pair(p0, config)?), p.as_str().to_string(), Some(PestPostOperator::from_pair(p1, config)?))),
            [(Rule::pest_pre_op, p0), (Rule::identifier, p)] => Ok(Self::Id(Some(PestPreOperator::from_pair(p0, config)?), p.as_str().to_string(), None)),
            [(Rule::pest_pre_op, p0), (Rule::identifier, p), (Rule::pest_post_op, p1)] => 
                Ok(Self::Id(Some(PestPreOperator::from_pair(p0, config)?), p.as_str().to_string(), Some(PestPostOperator::from_pair(p1, config)?))),
            [(Rule::pest_pre_op, p0), (Rule::left_bracket, _), (Rule::pest_expression, p), (Rule::right_bracket, _)] |
            [(Rule::pest_pre_op, p0), (Rule::pest_expression, p)] => 
                Ok(Self::Expression(Some(PestPreOperator::from_pair(p0, config)?), Box::new(PestExpression::from_pair(p, config)?), None)),
            [(Rule::pest_pre_op, p0), (Rule::left_bracket, _), (Rule::pest_expression, p), (Rule::right_bracket, _), (Rule::pest_post_op, p1)] => 
                Ok(Self::Expression(Some(PestPreOperator::from_pair(p0, config)?), Box::new(PestExpression::from_pair(p, config)?), Some(PestPostOperator::from_pair(p1, config)?))),
            [(Rule::pest_pre_op, p0), (Rule::pest_function, p)] => 
                Ok(Self::Function(Some(PestPreOperator::from_pair(p0, config)?), PestFunction::from_pair(p, config)?, None, None)),
            [(Rule::pest_pre_op, p0), (Rule::pest_function, p), (Rule::pest_post_op, p1)] => 
                Ok(Self::Function(Some(PestPreOperator::from_pair(p0, config)?), PestFunction::from_pair(p, config)?, None, Some(PestPostOperator::from_pair(p1, config)?))),
            [(Rule::sq_string, p), (Rule::sq_string, p1)] => 
                Ok(Self::Range(p.as_str().to_string(), p1.as_str().to_string())),
            _ => {
                error_string(format!("cannot process the following rule sequence: {pairs:?}"));
                Err(PestLangError::MissingCase(format!("missed case in PestAtom: {pairs:?}")))
            }
        }
    }

    pub fn get_post_opeartor(&self) -> Option<&PestPostOperator> {
        match self {
            Self::Expression(_, _, n) |
            Self::Function(_, _, _, n) |
            Self::Id(_, _, n) |
            Self::String(_, _, n) => n.as_ref(),
            Self::Range(_, _) => None,
        }
    }

    pub fn is_expression(&self) -> bool {
        match self {
            Self::Expression(_, _, _) => true,
            _ => false,
        }
    }

    pub fn get_expression(&self) -> Option<&PestExpression> {
        match self {
            Self::Expression(_, n, _) => Some(n),
            _ => None,
        }
    }

    pub fn get_expression_mut(&mut self) -> Option<&mut PestExpression> {
        match self {
            Self::Expression(_, n, _) => Some(n),
            _ => None,
        }
    }

    pub fn get_id(&self) -> Option<&str> {
        match self {
            Self::Id(_, id, _) => Some(id),
            _ => None,
        }
    }

    pub fn can_generate(s:&str) -> bool {
        !Self::is_reserved_keyword(s) && 
        !s.starts_with("\"") 
    }
    
    pub fn is_keyword(&self) -> bool {
        match self {
            Self::Id(_, id, _) => Self::is_reserved_keyword(id),
            _ => false,
        }
    }
    
    fn is_reserved_keyword(s:&str) -> bool {
        match s {
            "SOI" => true,
            "EOI" => true,
            "ANY" => true,
            "ASCII_DIGIT" => true,
            "ASCII_NONZERO_DIGIT" => true,
            "ASCII_BIN_DIGIT" => true,
            "ASCII_OCT_DIGIT" => true,
            "ASCII_HEX_DIGIT" => true,
            "ASCII_ALPHA" => true,
            "ASCII_ALPHANUMERIC" => true,
            "ASCII" => true,
            "NEWLINE" => true,
            "WHITESPACE" => true,
            _ => false,
        }
    }

    pub fn is_start_or_end_of_input(&self) -> bool {
        match self {
            Self::Id(_, n, _) => {
                if n == "SOI" || n == "EOI" {
                    return true;
                }
                false
            }
            _ => false,
        }
    }

    pub fn is_id(&self) -> bool {
        match self {
            Self::Id(_, _, _) => true,
            _ => false,
        }
    }

    pub fn is_string(&self) -> bool {
        match self {
            Self::String(_, _, _) => true,
            _ => false,
        }
    }

    pub fn can_generate_type(&self, rules:&PestRuleList) -> bool {
        match self {
            Self::Id(_, _, _) if self.is_start_or_end_of_input() => false,
            Self::Id(_, _, _) if self.is_keyword() && !self.is_start_or_end_of_input() => true,
            Self::Id(_, id, _) => {
                if let Some(v) = rules.can_generate_type(id) {
                    v
                }
                else {
                    panic!("cannot find the rule '{id}' in rules");
                }
            }
            Self::String(_, _, _) => false,
            Self::Expression(_, expr, _) => {
                expr.compute_can_generate_type(rules)
            }
            Self::Function(_, _, _, _) => false,
            Self::Range(_, _) => false,
        }
    }

    pub fn set_expression_tag(&mut self, tag:&str) {
        match self {
            Self::Expression(_, n, _) => n.set_sub_expression_tag(),
            _ => {}
        }
    }
}

#[derive(Debug, Clone)]
pub enum PestPreOperator {
    Bang,
    CaseInsensitive,
}

impl PestPreOperator {
    pub fn from_pair(pair:&Pair<Rule>, config:&mut CodeGenConfiguration) -> Result<Self, PestLangError> { 
        assert!(pair.as_rule() == Rule::pest_pre_op);

        match pair.as_str() {
            "!" => Ok(Self::Bang),
            "^" => Ok(Self::CaseInsensitive),
            _ => Err(PestLangError::MissingCase(format!("PestPreOperator: Miss case {pair:?}"))),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PestOperator {
    Or, 
    And,
}

impl PestOperator {
    pub fn from_pair(pair:&Pair<Rule>, config:&mut CodeGenConfiguration) -> Result<Self, PestLangError> { 
        let rr = pair.as_rule();
        assert!(rr == Rule::pest_binary_op || rr == Rule::pest_and || rr == Rule::pest_or);
        let s = pair.as_str();
        match s {
            "|" => Ok(Self::Or),
            "~" => Ok(Self::And),
            _ => Err(PestLangError::MissingCase(format!("cannot handle the case {s} for a PestOpeartor"))),
        }
    }
}

#[derive(Debug, Clone)]
pub enum PestPostOperator {
    Star,
    Plus,
    QuestionMark,
    Repeat(u32),
    Range(u32, u32),
}

impl PestPostOperator {
    pub fn from_pair(pair:&Pair<Rule>, config:&mut CodeGenConfiguration) -> Result<Self, PestLangError> { 
        assert!(pair.as_rule() == Rule::pest_post_op);

        let s = pair.as_str();
        match s {
            "*" => Ok(Self::Star),
            "+" => Ok(Self::Plus),
            "?" => Ok(Self::QuestionMark),
            _ => {
                let inner = pair.to_owned().into_inner();
                let pairs = inner.map(|x| (x.as_rule(), x)).collect::<Vec<_>>();
                match pairs.as_slice() {
                    [(Rule::integer, p)] => {
                        let v = u32::from_str_radix(p.as_str(), 10).map_err(|_| PestLangError::General(format!("cannot convert {p:?} to u32")))?;
                        Ok(Self::Repeat(v))
                    }
                    [(Rule::integer, p), (Rule::integer, p1)] => {
                        let v = u32::from_str_radix(p.as_str(), 10).map_err(|_| PestLangError::General(format!("cannot convert {p:?} to u32")))?;
                        let v1 = u32::from_str_radix(p1.as_str(), 10).map_err(|_| PestLangError::General(format!("cannot convert {p1:?} to u32")))?;
                        Ok(Self::Range(v, v1))
                    }
                    _ => Err(PestLangError::MissingCase(format!("Pest PostOperator: Missed {pair:?} in pest post operator")))
                }
            }
        }
    }
}

impl ToPestText for PestAtom {
    fn to_pest_text(&self) -> String {
        match self {
            Self::String(pre, s, post) => {
                let mut text = String::new();
                if let Some(pre_op) = pre {
                    text.push_str(&pre_op.to_pest_text());
                }
                text.push('"');
                text.push_str(s);
                text.push('"');
                if let Some(post_op) = post {
                    text.push_str(&post_op.to_pest_text());
                }
                text
            },
            Self::Id(pre, id, post) => {
                let mut text = String::new();
                if let Some(pre_op) = pre {
                    text.push_str(&pre_op.to_pest_text());
                }
                text.push_str(id);
                if let Some(post_op) = post {
                    text.push_str(&post_op.to_pest_text());
                }
                text
            }
            Self::Expression(pre, expr, post) => expr.to_pest_text(),
            Self::Function(pre, func, atom, post) => func.to_pest_text(),
            Self::Range(start, end) => format!("{}..{}", start, end),
        }
    }
}

impl ToPestText for PestPreOperator {
    fn to_pest_text(&self) -> String {
        match self {
            Self::Bang => "!".to_string(),
            Self::CaseInsensitive => "^".to_string(),
        }
    }
}

impl ToPestText for PestOperator {
    fn to_pest_text(&self) -> String {
        match self {
            Self::Or => "|".to_string(),
            Self::And => "~".to_string(),
        }
    }
}

impl ToPestText for PestPostOperator {
    fn to_pest_text(&self) -> String {
        match self {
            Self::Star => "*".to_string(),
            Self::Plus => "+".to_string(),
            Self::QuestionMark => "?".to_string(),
            Self::Repeat(n) => n.to_string(),
            Self::Range(start, end) => format!("{}..{}", start, end),
        }
    }
}