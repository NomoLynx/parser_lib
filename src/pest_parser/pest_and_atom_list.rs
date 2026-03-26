use std::ops::Index;

use super::{pest_atom::PestAtom, pest_rule_list::PestRuleList, traits::to_pest_text::ToPestText};

#[derive(Debug, Clone)]
pub struct PestAndAtomList<'a> {
    atoms : Vec<&'a PestAtom>,
    name: String,
    id: i32,
}

impl<'a> PestAndAtomList<'a> {
    pub fn new(atoms:Vec<&'a PestAtom>, name: String, id: i32) -> Self {
        PestAndAtomList { atoms, name, id }
    }

    pub fn get_atoms(&self) -> &Vec<&PestAtom> {
        &self.atoms
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn get_unique_name(&self) -> String {
        format!("{}_{:04}", self.name, self.id)
    }

    pub fn can_generate_type(&self, rules:&PestRuleList) -> bool {
        for atom in &self.atoms {
            if atom.can_generate_type(rules) {
                return true;
            }
        }

        false
    }

    pub fn len(&self) -> usize {
        self.atoms.len()
    }
}

impl<'a> Index<usize> for PestAndAtomList<'a> {
    type Output = &'a PestAtom;

    fn index(&self, index: usize) -> &Self::Output {
        &self.atoms[index]
    }
}

impl ToPestText for PestAndAtomList<'_> {
    fn to_pest_text(&self) -> String {
        let mut r = String::new();
        for atom in &self.atoms {
            r.push_str(&atom.to_pest_text());
        }
        r
    }
}