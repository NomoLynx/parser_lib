
use std::collections::HashMap;

use pest::iterators::Pair;

use super::{code_gen_config::CodeGenConfiguration, pest_and_atom_list::PestAndAtomList, pest_atom::*, pest_lang_err::PestLangError, pest_pest::Rule, pest_rule_list::PestRuleList, traits::to_pest_text::ToPestText};

#[derive(Debug, Clone)]
pub struct PestExpression {
    left : PestAtom,
    rights : Vec<(PestOperator, PestAtom)>,
    tag : String,

    // expression properties
    can_generate_type : bool,
    unique_id : i32,
}

impl PestExpression {
    pub fn from_pair(pair:&Pair<Rule>, config:&mut CodeGenConfiguration) -> Result<Self, PestLangError> {
        let inner = pair.to_owned().into_inner();
        let pairs = inner.map(|x| (x.as_rule(), x)).collect::<Vec<_>>();
        match pairs.as_slice() {
            [(Rule::pest_or, _), (Rule::pest_atom, p)] |
            [(Rule::pest_atom, p)] => 
                Ok(Self { left : PestAtom::from_pair(p, config)?, rights : Vec::default(), tag : String::new(), 
                    can_generate_type : true, unique_id : -1 }),
            [(Rule::pest_or, _), (Rule::pest_atom, p), rest@..] |
            [(Rule::pest_atom, p), rest@..] => {
                let left = PestAtom::from_pair(p, config)?;
                let (a, b) : (Vec<_>, Vec<_>)= rest.into_iter().partition(|x| x.0 == Rule::pest_and || x.0 == Rule::pest_or);
                assert!(a.len() == b.len());

                let r = a.iter().zip(b.iter())
                 .map(|(x,y) | (PestOperator::from_pair(&x.1, config).unwrap(), PestAtom::from_pair(&y.1, config).unwrap()) )
                 .collect::<Vec<_>>();

                Ok(Self { left, rights : r, tag: String::new(), can_generate_type : true, unique_id : -1, 
                })
            }
            _ => Err(PestLangError::MissingCase(format!("Missed case in PestExpression {pairs:?}"))),
        }
    }

    pub fn get_all_atoms(&self) -> Vec<&PestAtom> {
        let mut r = vec![&self.left];
        r.append(&mut self.rights.iter().map(|(_,b)| b).collect::<Vec<_>>() );
        r
    }

    pub fn get_all_atoms_mut(&mut self) -> Vec<&mut PestAtom> {
        let mut r = vec![&mut self.left];
        r.append(&mut self.rights.iter_mut().map(|(_, b)| b).collect::<Vec<_>>() );
        r
    }

    pub fn get_name(&self) -> String {
        self.get_tag().to_string()
    }

    pub fn get_tag(&self) -> &str {
        self.tag.as_ref()
    }

    pub fn set_tag(&mut self, value:&str) {
        self.tag = value.to_string();
    }

    pub fn get_type_name(&self) -> String {
        to_pascal_case(&self.tag)
    }

    /// get pest atom list list (each element is a pest atom) by the operator
    pub fn get_atom_list_by(&self, operator:&PestOperator) -> Vec<PestAndAtomList<'_>> {
        let mut r = Vec::default();
        let mut init = vec![&self.left];

        let mut id = 0;
        for (op, atom) in &self.rights {
            if op != operator {
                init.push(atom);
            }
            else {
                r.push(PestAndAtomList::new(init.clone(), self.get_name(), id));
                id += 1;
                init.clear();
                init.push(atom);
            }
        }

        if !init.is_empty() {
            r.push(PestAndAtomList::new(init.clone(), self.get_name(), id));
            init.clear();
        }

        r
    }

    pub fn get_unique_id(&self) -> i32 {
        self.unique_id
    }

    pub fn set_can_generate_type(&mut self, value:bool) {
        self.can_generate_type = value;
    }

    pub fn get_can_generate_type(&self) -> bool {
        self.can_generate_type
    }

    pub fn set_unique_id_deep(&mut self, id:&mut i32) {
        if self.get_unique_id() != -1 {
            return;  //means it's already set
        }

        self.unique_id = *id;
        *id += 1;
        
        for atom in self.get_all_atoms_mut() {
            if let Some(expr) = atom.get_expression_mut() {
                expr.set_unique_id_deep(id);
            }
        }
    }

    pub fn compute_can_generate_type(&self, rules:&PestRuleList) -> bool {
        let v = self.get_all_atoms().iter()
                  .any(|x| x.can_generate_type(rules));
        v
    }

    /// set child expression's tag
    pub fn set_sub_expression_tag(&mut self) {
        let tag = self.get_tag().to_string();
        for (i, atom) in self.get_all_atoms_mut().into_iter().enumerate() {
            if let Some(expr) = atom.get_expression_mut() {
                let new_tag = format!("{}_{i}", tag);
                expr.set_tag(&new_tag);
                expr.set_sub_expression_tag();
            }
        }
    }

    /// get expressin list from current expression
    pub fn finalize_can_generate_type(&mut self, mapping:&HashMap<String, bool>) {
        let mut r = false;
        for atom in self.get_all_atoms_mut() {
            r |= match atom { 
                PestAtom::Function(_, _, _, _) |
                PestAtom::Range(_, _) |
                PestAtom::String(_, _, _) => false,
                PestAtom::Id(_, _, _) if atom.is_start_or_end_of_input() => false, 
                PestAtom::Id(_, _, _) if atom.is_keyword() => true, 
                PestAtom::Id(_, id, _) => mapping[id],
                PestAtom::Expression(_, expr, _) => {
                    expr.finalize_can_generate_type(mapping);
                    expr.get_can_generate_type()
                }
            }
        }

        self.set_can_generate_type(r);
    }
}

impl ToPestText for PestExpression {
    fn to_pest_text(&self) -> String {
        let mut r = String::new();
        r.push_str(&self.left.to_pest_text());
        for (op, atom) in &self.rights {
            r.push_str(&op.to_pest_text());
            r.push_str(&atom.to_pest_text());
        }
        r
    }
}

fn to_pascal_case(value: &str) -> String {
    value
        .split(|c: char| !c.is_alphanumeric())
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str().to_ascii_lowercase()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join("")
}