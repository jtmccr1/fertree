use super::newick_parser::NewickParser;
use crate::tree::mutable_tree::MutableTree;
use std::error::Error;
use std::fs::File;
use std::io;
use std::io::{BufRead, Read};
use std::path::PathBuf;
use crate::io::error::IoError;

//https://stackoverflow.com/questions/36088116/how-to-do-polymorphic-io-from-either-a-file-or-stdin-in-rust/49964042
pub struct NewickImporter<'a> {
    source: Box<dyn BufRead + 'a>,
}
impl<'a> NewickImporter<'a> {
    pub fn from_console(stdin: &'a io::Stdin) -> NewickImporter<'a> {
        NewickImporter {
            source: Box::new(stdin.lock()),
        }
    }
    pub fn from_path(path: PathBuf) -> io::Result<NewickImporter<'a>> {
        File::open(path).map(|file| NewickImporter {
            source: Box::new(io::BufReader::new(file)),
        })
    }
}

impl<'a> Read for NewickImporter<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.source.read(buf)
    }
}

impl<'a> BufRead for NewickImporter<'a> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.source.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.source.consume(amt);
    }
}

impl<'a> Iterator for NewickImporter<'a> {
    type Item = Result<MutableTree,IoError>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(line) = self.lines().next() {
            match line {
                Ok(nwk_string) => {
                    let tree = NewickParser::parse_string(nwk_string.as_bytes());
                    match tree {
                        Ok(node) => Some(Ok(node)),
                        Err(e) => Some(Err(e)),

                    }
                }
                Err(e) => Some(Err(IoError)),
            }
        } else {
            None
        }
    }
}

