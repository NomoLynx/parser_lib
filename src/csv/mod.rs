pub mod csv_pest;
pub mod csv_error;
pub mod csv_type;
pub mod csv_conversion;

pub use csv_pest::*;
pub use csv_type::*;
pub use csv_conversion::*;

use pest::Parser;

use crate::{common::*, csv::csv_error::CSVError};

pub struct CodeGenConfiguration;

pub fn parse_csv_file(file_path: &str) -> Result<CSVFile, CSVError> {
    let file_content = read_file_content(file_path)
        .map_err(|e| CSVError::IoError(e))?;
    parse_csv(&file_content)
}

pub fn parse_csv(input: &str) -> Result<CSVFile, CSVError> {
    let mut config=CodeGenConfiguration;

    let mut pairs = CSVParser::parse(Rule::csv, input)
        .map_err(|e| CSVError::InvalidFormat(format!("{e:?}")))?;
    
    if let Some(pair) = pairs.find(|n| n.as_str().len() == input.len()) {
        let prog_r = CSVFile::from_pair(&pair,&mut config);
        if prog_r.is_err() {
            error_str("cannot get program from Rule::START");
            error_string(format!("CSV parsing error: {:?}", prog_r.err()));
            Err(CSVError::GeneralError("cannot get program from Rule::START".to_string()))
        } else {
            let prog = prog_r.unwrap();
            Ok(prog)
        }
    } else {
        error_string(format!(
            "Error: {} at {}",
            "does not catch all string",
            input.to_owned()
        ));
        debug_string(format!("input: {}\r\nParsed: {:#?}", input, pairs));
        let count = pairs.count();
        debug_string(format!("Pairs count = {count}\r\n"));
        Err(CSVError::GeneralError("cannot parse all input string".to_string()))
    }  
}

/// Converts a 1-based integer to its corresponding Excel column name.
pub fn int_to_excel_col(n: usize) -> String {
    assert!(n >= 1, "Excel columns are 1-based");

    let mut x = n;
    let mut s = String::new();

    while x > 0 {
        let r = (x - 1) % 26;
        s.push((b'A' + r as u8) as char);
        x = (x - 1) / 26;
    }

    s.chars().rev().collect()
}

pub fn validate_csv_column_value_unique_from_file(file_path: &str, column_name: &str) -> bool {
    match parse_csv_file(file_path) {
        Ok(csv_file) => validate_csv_column_value_unique(&csv_file, column_name),
        Err(e) => {
            error_string(format!("Failed to parse CSV file: {:?}", e));
            false
        }
    }
}

pub fn validate_csv_column_value_unique(csv_file: &CSVFile, column_name: &str) -> bool {
    if let Some(col_index) = csv_file.find_column_index_by_name(column_name) {
        let mut value_set = std::collections::HashSet::new();
        for data in csv_file.get_column_data(col_index) {
            if !value_set.insert(data.to_string()) {
                return false; // Duplicate found
            }
        }
        true // All values are unique
    } else {
        false // Column not found
    }
}