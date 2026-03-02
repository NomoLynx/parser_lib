use std::collections::HashMap;

use crate::common::*;

pub mod init_file_pest;
pub mod type1type;
pub mod property;
pub mod section_data;

pub use init_file_pest::*;
pub use type1type::*;
pub use property::*;
pub use section_data::*;

pub fn get_ini_properties(ini_file : &InitFile) -> HashMap<String, String> {
    let mut r = ini_file.get_first_layer_properties()
                                                    .iter()
                                                    .map(|x| x.to_tuple())
                                                    .map(|(a, b)| (a.trim().to_string(), b))
                                                    .collect::<HashMap<_,_>>();

    let other = ini_file.get_items().iter()
                                        .filter_map(|x| x.get_section_data())
                                        .collect::<Vec<_>>();

    for section_data in other {
        let prefix = section_data.get_section();
        for property in section_data.get_property() {
            let name = format!("{prefix}_{}", property.get_name()).trim().to_string();
            let value = format!("{}", property.get_value());
            r.insert(name, value);
        }
    }

    r
}

pub fn get_asm_data_code_from_ini(ini_file : &InitFile) -> String {
    let items = get_ini_properties(ini_file);
    let asm_statements = items.iter().map(|(name, value)| format!("{name}:   .string \"{value}\"")).collect::<Vec<_>>();

    let r = format!(".data\r\n\r\n{}", asm_statements.join("\r\n"));
    r
}

pub fn parse_ini_from_file(file_path:&str) -> Result<InitFile, InitFileError> {
    match read_file_content(file_path) {
        Ok(input) => {
            let ini_file = InitFile::parse(& input)?;
            Ok(ini_file)
        }
        Err(e) => {
            error_string(format!("parse ini from file error: {e:?}"));
            Err(InitFileError::FileError)
        }
    }
}

pub fn ini_file_to_asm_data_code(file_path:&str) -> Result<String, InitFileError> {
    match read_file_content(file_path) {
        Ok(input) => {
            let ini_file = InitFile::parse(& input)?;
            let r = get_asm_data_code_from_ini(& ini_file);
            Ok(r)
        }
        Err(e) => {
            error_string(format!("ini file to asm data error: {e:?}"));
            Err(InitFileError::FileError)
        }
    }
}

pub type IniFileProperties = (Name, Vec<Property>);

/// get ini location, name and properties from ini file
pub fn get_ini_location_and_properties(file_path:&str) -> Result<IniFileProperties, InitFileError> {
    let input = read_file_content(file_path)
        .map_err(|_| InitFileError::FileError)?;

    let ini_file = InitFile::parse(& input)?;
    let file_name = file_path.to_string();

    let properties = ini_file.get_first_layer_properties()
                            .iter()
                            .cloned()
                            .cloned()
                            .collect::<Vec<_>>();

    Ok( (file_name, properties) )
}