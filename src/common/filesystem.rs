use std::path::Path;

use crate::common::*;
use core_utils::filesystem as core_filesystem;

pub fn file_exists<S>(file_path: S) -> bool
    where S : AsRef<str> {
    core_filesystem::file_exists(file_path)
}

pub fn path_file_exists(folder:&str, file:&str) -> bool {
    core_filesystem::path_file_exists(folder, file)
}

pub fn folder_exists(path:&str) -> bool {
    core_filesystem::folder_exists(path)
}

pub fn read_file_to_string<S>(path: S) -> String
    where S : AsRef<str> {
    core_filesystem::read_file_to_string(path)
}

pub fn read_file_content<P:AsRef<Path>>(file_path: P) -> std::io::Result<String> {
    core_filesystem::read_file_content(file_path)
}

pub fn write_to_file(file_name:&str, content:&str) -> Result<(), ParsingError> {
    core_filesystem::write_to_file(file_name, content).map_err(|_| ParsingError::IOError)?;
    Ok(())
}

pub fn write_to_file_option(file_name_option:Option<&String>, content:&str) -> Result<(), ParsingError> {
    if let Some(file_name) = file_name_option {
        write_to_file(file_name, content)
    }
    else {
        debug_string(format!("{content}"));
        Ok(())
    }
}

pub fn append_to_file(file_name:&str, content:&str) -> Result<(), ParsingError> {
    core_filesystem::append_to_file(file_name, content).map_err(|_| ParsingError::IOError)?;
    Ok(())
}

pub fn append_to_file_option(file_name_option:Option<&String>, content:&str) -> Result<(), ParsingError> {
    if let Some(file_name) = file_name_option {
        append_to_file(file_name, content)
    }
    else {
        debug_string(format!("{content}"));
        Ok(())
    }
}

pub fn delete_file_option(file_name_option:Option<&String>) -> Result<(), ParsingError> {
    core_filesystem::delete_file_option(file_name_option).map_err(|_| ParsingError::IOError)?;
    Ok(())
}