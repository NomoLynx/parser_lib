#[derive(Debug, Clone)]
pub enum PestLangError {
    General(String),
    MissingCase(String),
}