use crate::io::error::IoError;
use crate::tree::mutable_tree::MutableTree;

pub trait TreeImporter<R>: Iterator {
    fn has_tree(&mut self) -> bool;
    fn read_next_tree(&mut self) -> Result<MutableTree, IoError>;
    fn skip_tree(&mut self);
}
