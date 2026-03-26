use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::{pest_pest::PestRule, traits::to_pest_text::ToPestText};

pub struct PestRuleList {
    data: Vec<Rc<RefCell<PestRule>>>,
    can_generate_type_mapping: HashMap<String, bool>,
}

impl PestRuleList {

    pub fn get_data(&self) -> &Vec<Rc<RefCell<PestRule>>> {
        &self.data
    }

    pub fn set_expression_unique_id(&self) {
        let mut id = 0;
        for rule in self.data.iter() {
            let mut rule_borrow = rule.borrow_mut();
            let expr = rule_borrow.get_expression_mut();
            expr.set_unique_id_deep(&mut id);
            id += 1;
        }
    }

    pub fn set_expression_tag(&mut self) {
        for rule in self.data.iter() {
            let mut rule_borrow = rule.borrow_mut();
            rule_borrow.set_expression_tag();
        }
    }

    pub fn set_expression_can_generate_type(&mut self) {
        // init mapping
        for rule in self.data.iter() {
            let name = rule.borrow().get_name();
            self.can_generate_type_mapping.insert(name, true);
        }
        
        let mut changed;
        loop {
            changed = false;
            for rule in self.data.iter() {
                let can_gen = rule.borrow().compute_can_generate_type(self);
                if rule.borrow().get_can_generate_type() != can_gen {
                    changed = true;
                    rule.borrow_mut().set_can_generate_type(can_gen);
                }
            }

            if !changed {
                break;
            }
        }

        // build up the mapping from final results
        for rule in self.data.iter() {
            let name = rule.borrow().get_name();
            let can_gen = rule.borrow().get_can_generate_type();
            self.can_generate_type_mapping.insert(name, can_gen);
        }

        // perform the final computation so all sub-expressions are set correctly
        let mapping = self.can_generate_type_mapping.clone();
        for rule in self.data.iter() {
            rule.borrow_mut().finalize_can_generate_type(&mapping);
        }
    }

    pub fn can_generate_type(&self, name:&str) -> Option<bool> {
        self.can_generate_type_mapping.get(name).cloned()
    }

    pub fn generate_types(&self) {
        // Kept as a compatibility no-op. Type generation lives in CLR codegen.
    }

    pub fn get_can_generate_type_mapping(&self) -> &HashMap<String, bool> {
        &self.can_generate_type_mapping
    }
}

impl From<Vec<PestRule>> for PestRuleList {
    fn from(v: Vec<PestRule>) -> Self {
        PestRuleList { 
            data: v.into_iter()
                .map(|rule| Rc::new(RefCell::new(rule)))
                .collect(),
            can_generate_type_mapping: HashMap::new(),
         }
    }
}

impl ToPestText for PestRuleList {
    fn to_pest_text(&self) -> String {
        let mut r = String::new();
        for rule in self.data.iter() {
            r.push_str(&rule.borrow().to_pest_text());
            r.push_str("\n");
        }
        r
    }
}