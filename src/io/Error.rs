use std::fmt;

#[derive(Debug, Clone)]
pub struct IoError;

impl fmt::Display for IoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "io error")
    }
}
