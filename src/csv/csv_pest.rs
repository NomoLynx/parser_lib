use pest::iterators::Pair;
use pest_derive::Parser;
use rust_macro::*;
use crate::{common::*, csv::{CodeGenConfiguration, csv_error::CSVError}};

#[derive(Parser)]
#[grammar = "csv/csv.pest"]
pub struct CSVParser;

#[derive(Debug, Accessors)]
pub struct CSVFile {
    records: Vec<CSVRecord>,
}

impl CSVFile {
    pub fn new(records: Vec<CSVRecord>) -> Self {
        CSVFile { records }
    }

    pub fn from_pair(pair: &Pair<Rule>, config:&CodeGenConfiguration) -> Result<Self, CSVError> {
        assert!(pair.as_rule() == Rule::csv);
        let mut records = Vec::new();

        for record_pair in pair.to_owned().into_inner() {
            match record_pair.as_rule() {
                Rule::record => {
                    records.push(CSVRecord::from_pair(&record_pair, config)?);
                }
                Rule::EOI => {}
                _ => {
                    return Err(CSVError::InvalidFormat(format!("CSVFile unexpected rule: {:?}", record_pair.as_rule())));
                }
            }
        }

        // if the last few records are empty, we will ignore them, which is a common case when the csv file has empty lines at the end
        while let Some(last_record) = records.last() {
            if last_record.fields.is_empty() || last_record.fields.iter().all(|f| f.trim().is_empty()) {
                records.pop();
            } else {
                break;
            }
        }

        Ok(CSVFile::new(records))
    }

    /// Get the number of columns in the CSV file
    pub fn get_column_count(&self) -> usize {
        if let Some(first_record) = self.records.first() {
            first_record.fields.len()
        } else {
            0
        }
    }

    /// Get a specific column by index
    pub fn get_column(&self, index: usize) -> Vec<&str> {
        self.records
            .iter()
            .filter_map(|record| record.fields.get(index).map(|s| s.as_str()))
            .collect()
    }

    /// get a specific column by index, ignore the first element assuming it's the header
    pub fn get_column_data(&self, index: usize) -> Vec<&str> {
        self.records
            .iter()
            .skip(1)
            .filter_map(|record| record.fields.get(index).map(|s| s.as_str()))
            .collect()
    }

    /// get data records, which is ignore the 1st record as header
    pub fn get_data_records(&self) -> Vec<&CSVRecord> {
        self.records.iter().skip(1).collect()
    }

    /// get header names, which is 1st row in the csv file
    pub fn get_header_names(&self) -> Vec<&str> {
        if let Some(first_record) = self.records.first() {
            first_record.fields.iter().map(|s| s.as_str()).collect()
        } else {
            vec![]
        }
    }

    /// get header name by column index
    pub fn get_header_name(&self, index: usize) -> Option<String> {
        self.get_header_names().get(index)
            .map(|s| s.to_string())
    }

    /// find column index by column name
    pub fn find_column_index_by_name(&self, name:&str) -> Option<usize> {
        if let Some(first_record) = self.records.first() {
            for (i, field) in first_record.fields.iter().enumerate() {
                if field == name {
                    return Some(i);
                }
            }
        }

        None
    }

    /// get data in a column by column name
    pub fn get_column_by_name(&self, name:&str) -> Option<Vec<&str>> {
        if let Some(index) = self.find_column_index_by_name(name) {
            Some(self.get_column_data(index))
        }
        else {
            None
        }
    }
    
    /// get cell value by row index and column index
    pub fn get_cell_value(&self, row_index: usize, col_index: usize) -> String {
        if let Some(record) = self.records.get(row_index) {
            if let Some(field) = record.fields.get(col_index) {
                return field.clone();
            }
        }

        String::new()
    }
}

#[derive(Debug, Accessors)]
pub struct CSVRecord {
    fields: Vec<String>,
}

impl CSVRecord {
    pub fn new(fields: Vec<String>) -> Self {
        CSVRecord { fields }
    }

    pub fn from_pair(pair: &Pair<Rule>, config:&CodeGenConfiguration) -> Result<Self, CSVError> {
        assert!(pair.as_rule() == Rule::record);

        let mut fields = Vec::new();
        for field_pair in pair.to_owned().into_inner() {
            match field_pair.as_rule() {
                Rule::unquoted_field | Rule::quoted_field => {
                    let field = CSVField::from_pair(&field_pair, config)?;
                    fields.push(field.as_string().to_string());
                }
                _ => {
                    return Err(CSVError::InvalidFormat(format!("CSVRecord unexpected rule: {:?}", field_pair.as_rule())));
                }
            }
        }

        Ok(CSVRecord::new(fields))
    }

    pub fn get_field(&self, i:usize) -> &str {
        &self.fields[i]
    }
}

#[derive(Debug, EnumAccessors)]
pub enum CSVField {
    Quoted(String),
    Unquoted(String),
}

impl CSVField {
    pub fn from_pair(pair: &Pair<Rule>, _config:&CodeGenConfiguration) -> Result<Self, CSVError> {
        assert!(pair.as_rule() == Rule::quoted_field || pair.as_rule() == Rule::unquoted_field);
        
        match pair.as_rule() {
            Rule::quoted_field => {
                let str = strip_str(strip_quotes(pair.as_str()), "\\\"");
                Ok(CSVField::Quoted(str.to_string()))
            }
            Rule::unquoted_field => Ok(CSVField::Unquoted(pair.as_str().to_string())),
            _ => Err(CSVError::InvalidFormat(format!("Unexpected rule: {:?}", pair.as_rule()))),
        }
    }

    pub fn as_string(&self) -> &String {
        match self {
            CSVField::Quoted(s) => s,
            CSVField::Unquoted(s) => s,
        }
    }
}