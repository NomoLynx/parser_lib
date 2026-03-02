use pest::iterators::Pair;
use rust_macro::*;

#[derive(Debug, Clone, Accessors)]
pub struct Location {
    line: usize,
    column: usize,
}

impl<'i, R: pest::RuleType> From<&'i Pair<'i, R>> for Location {
    fn from(pair: &Pair<'i, R>) -> Self {
        let span = pair.as_span();
        let (line, column) = span.start_pos().line_col();
        Location { line, column }
    }
}