use pest::error::LineColLocation;

#[derive(Debug, Clone)]
pub enum MermaidError {
    InvalidPacketDefinition,
    GeneralError(String),
    FileNotFound(String),
    PestParsingError(String, usize, usize),
    ParsingConversionError,
    FileError,
}

impl MermaidError {
    pub(crate) fn get_location_from_pest_input_location(
        str: &str,
        input_location: &LineColLocation,
    ) -> Self {
        let pos = match input_location {
            LineColLocation::Pos((a, b)) => (a, b),
            LineColLocation::Span((a, b), _) => (a, b),
        };

        Self::PestParsingError(str.to_string(), *pos.0, *pos.1)
    }
}