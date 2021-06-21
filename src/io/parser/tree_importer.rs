use crate::tree::mutable_tree::MutableTree;
use crate::io::error::IoError;

pub trait TreeImporter<R> :Iterator<Item=MutableTree>{
    fn has_tree(&mut self)->bool;
    fn read_next_tree(&mut self)->Result<MutableTree,IoError>;
}
