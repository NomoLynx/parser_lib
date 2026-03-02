#[derive(Debug)]
pub enum CSVError {
    InvalidFormat(String),
    IoError(std::io::Error),
    GeneralError(String),
}