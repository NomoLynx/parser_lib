pub mod markdown_err;
pub mod markdown_pest;
pub mod markdown_pest_error;
pub mod string_value_table;

pub use markdown_err::*;
pub use markdown_pest::*;
pub use markdown_pest_error::*;
pub use string_value_table::*;

pub fn load_md_file(path_str: &str) -> Result<File, crate::common::ParsingError> {
    let md_file = markdown_pest::File::from_file(path_str)?;
    Ok(md_file)
}