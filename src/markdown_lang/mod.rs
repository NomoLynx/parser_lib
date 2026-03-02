pub mod markdown_pest;
pub mod markdown_pest_error;
pub mod markdown_err;
pub mod string_value_table;

pub use markdown_pest::*;
pub use markdown_pest_error::*;
pub use markdown_err::*;
pub use string_value_table::*;

pub fn load_md_file(path_str:&str) -> Result<File, crate::common::ParsingError> {
	let md_file = markdown_pest::File::from_file(path_str)?;
	Ok(md_file)
}

/// Convert a u32 number to a base-26 string (A-Z)
pub fn u32_to_base26(mut number: u32) -> String {
	if number == 0 {
		return "A".to_string();
	}

	let mut result = String::new();
	while number > 0 {
		let remainder = (number % 26) as u8;
		let ch = (b'A' + remainder) as char;
		result.push(ch);
		number /= 26;
	}

	result.chars().rev().collect()
}
