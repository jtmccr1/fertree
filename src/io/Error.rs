use std::fmt;
use std::error::Error;

#[derive(Debug, Clone)]
pub enum IoError{
    EOF,
    OTHER
}
impl Error for IoError{}
impl fmt::Display for IoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "io error")
    }
}
