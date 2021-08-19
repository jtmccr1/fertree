use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub enum IoError {
    Eof,
    Format(String),
    DuplicateTaxon(String),
    Other,
}
impl Error for IoError {}
impl fmt::Display for IoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "need a more informative error")
    }
}
