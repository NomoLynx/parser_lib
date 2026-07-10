pub mod ini;
pub mod mermaid_packet;
pub mod mermaid_flow;
pub mod csv;
pub mod common;
pub mod mermaid_error;
pub mod json;
pub mod mermaid_state;
pub mod mermaid_sequence;
pub mod markdown_lang;
pub mod expr_lang;
pub mod string_format;
pub mod pest_parser;

use core_utils::debug::error_string;
use pest::iterators::Pair;
use pest::RuleType;

pub (crate) fn error_rule_pair<R: RuleType>(rule:&R, p: &Pair<R>) {
    error_string(format!("{rule:?}, {p:?}"))
}

pub (crate) fn error_rule_pair_vec<R : RuleType>(l:&[(R, Pair<R>)]) {
    for (rule, p) in l {
        error_rule_pair(rule, p);
    }
}