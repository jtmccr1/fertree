use super::newick_parser::NewickParser;
use crate::tree::mutable_tree::{MutableTree, TreeIndex};
use std::error::Error;
use std::fs::File;
use std::io;
use std::io::{BufRead, Read, BufReader};
use std::path::PathBuf;
use crate::io::error::{IoError};
use std::collections::HashMap;
use crate::tree::AnnotationValue;
use crate::io::parser::annotation_parser::AnnotationParser;

type Result<T> = std::result::Result<T, IoError>;
type Byte = u8;

//https://stackoverflow.com/questions/36088116/how-to-do-polymorphic-io-from-either-a-file-or-stdin-in-rust/49964042
pub struct NewickImporter<R> {
    last_token: Option<u8>,
    tree: Option<MutableTree>,
    reader: Box<BufReader<R>>,
    last_deliminator: u8,
    last_annotation: Option<HashMap<String, AnnotationValue>>,
}

impl<R: std::io::Read> NewickImporter<R> {
    pub fn from_reader(reader:R)->Self{
        NewickImporter {
            last_token: None,
            //TODO better cleaner api through new.
            tree: None,
            reader: Box::new(BufReader::new(reader)),
            last_annotation: None,
            last_deliminator: b'\0',
        }
    }
    pub fn read_tree(input_reader: BufReader<R>) -> Result<MutableTree> {
        let mut parser = NewickImporter {
            last_token: None,
            //TODO better cleaner api through new.
            tree: None,
            reader: Box::new(input_reader),
            last_annotation: None,
            last_deliminator: b'\0',
        };

        parser.read_next_tree()
    }
    fn read_next_tree(&mut self) -> Result<MutableTree> {
        let start = std::time::Instant::now();
        self.tree = Some(MutableTree::new());
        self.skip_until(b'(')?;
        self.unread_token(b'(');

        let root = self.read_internal_node()?;
        //TODO hide node/node ref api
        self.get_tree().set_root(Some(root));
        self.get_tree().branchlengths_known = true;

        match self.last_deliminator {
            b')' => Err(IoError::OTHER),
            b';' => {
                trace!(
                    "Tree parsed in {} milli seconds ",
                    start.elapsed().as_millis()
                );
                Ok(self.tree.take().unwrap())
            }
            _ => Err(IoError::OTHER)
        }
    }
    fn read_internal_node(&mut self) -> Result<TreeIndex> {
        let token = self.read_token()?;
        //assert =='('
        let mut children = vec![];
        children.push(self.read_branch()?);

        // read subsequent children
        while self.last_deliminator == b',' {
            children.push(self.read_branch()?);
        }

        // should have had a closing ')'
        if self.last_deliminator != b')' {
            // throw new BadFormatException("Missing closing ')' in tree");
            Err(IoError::OTHER)
        } else {
            let label = self.read_to_token(",:();")?;
            let node = self.get_tree().make_internal_node(children);
            if !label.is_empty() {
                self.get_tree().label_node(node, label);
            }
            self.annotation_node(node);
            //TODO root branch length?
            Ok(node)
        }
    }
    fn read_external_node(&mut self) -> Result<TreeIndex> {
        let label = self.read_to_token(",:();")?;
        let node = self.get_tree().make_external_node(label.as_str(), None).expect("Failed to make tip");
        self.annotation_node(node);

        Ok(node)
    }
    fn read_branch(&mut self) -> Result<TreeIndex> {
        let mut length = 0.0;

        let branch = if self.next_token()? == b'(' {
            // is an internal node
            self.read_internal_node()?
        } else {
            // is an external node
            self.read_external_node()?
        };
        //TODO branch comments?

        if self.last_deliminator == b':' {
            length = self.read_double(",():;")?;
            self.annotation_node(branch);
        }

        self.get_tree().set_length(branch, length);

        Ok(branch)
    }
    fn unread_token(&mut self, c: Byte) {
        self.last_token = Some(c);
    }

    fn next_token(&mut self) -> Result<Byte> {
        match self.last_token {
            None => {
                let c = self.read_token()?;
                self.last_token = Some(c);
                Ok(c)
            }
            Some(c) => {
                Ok(c)
            }
        }
    }
    fn read_token(&mut self) -> Result<Byte> {
        self.skip_space()?;
        let mut ch = self.read()?;
        // while hasComments && (ch == startComment || ch == lineComment) {
        while ch == b'[' {
            self.skip_comments(ch)?;
            self.skip_space()?;
            ch = self.read()?;
        }

        Ok(ch)
    }

    fn read_to_token(&mut self, deliminator: &str) -> Result<String> {
        let delims = deliminator.bytes().collect::<Vec<Byte>>();
        let mut space = 0;
        let mut ch = b'\0';
        let mut ch2 = b'\0';
        let mut quote_char = b'\0';

        let mut done = false;
        let mut first = true;
        let mut quoted = false;

        self.next_token()?;
        let mut token = String::new();
        while !done {
            ch = self.read()?;
            let is_space = char::from(ch).is_whitespace();
            if quoted && ch == quote_char {
                ch2 = self.read()?;
                if ch == ch2 {
                    token.push(char::from(ch));
                } else {
                    // self.last_deliminator=' ';
                    self.unread_token(ch2);
                    // done=true;
                    quoted = false;
                }
            } else if first && (ch == b'\'' || ch == b'"') {
                quoted = true;
                quote_char = ch;
                first = false;
                space = 0;
            } else if ch == b'[' {
                self.skip_comments(ch)?;
                // self.last_deliminator=' ';
                done = true
            } else {
                if quoted {
                    if is_space {
                        space += 1;
                        ch = b' ';
                    } else {
                        space = 0;
                    }
                    if space < 2 {
                        token.push(char::from(ch));
                    }
                } else if is_space {
                    self.last_deliminator = b' ';
                    done = true;
                } else if delims.contains(&ch) {
                    done = true;
                    self.last_deliminator = ch;
                } else {
                    token.push(char::from(ch));
                    first = false;
                }
            }
        }
        if char::from(self.last_deliminator).is_whitespace() {
            ch = self.next_token()?;
            while char::from(ch).is_whitespace() {
                self.read()?;
                ch = self.next_token()?;
            }
            if !delims.contains(&ch) {
                self.last_deliminator = self.read_token()?;
            }
        }

        Ok(token)
    }

    fn read_double(&mut self, deliminator: &str) -> Result<f64> {
        let s = self.read_to_token(deliminator)?;
        //TODO capture this error

        match s.parse() {
            Ok(l) => Ok(l),
            Err(e) => Err(IoError::OTHER)
        }
    }

    fn read(&mut self) -> Result<Byte> {
        let mut buf: [u8; 1] = [0; 1];
        match self.last_token {
            None => {
                match self.reader.read(&mut buf) {
                    Ok(1) => Ok(buf[0]),
                    Ok(0) => Err(IoError::EOF),
                    _ => Err(IoError::OTHER)
                }
            }
            Some(c) => {
                self.last_token = None;
                Ok(c)
            }
        }
    }

    fn skip_space(&mut self) -> Result<()> {
        let mut ch: Byte = self.read()?;
        while char::from(ch).is_whitespace() {
            ch = self.read()?;
        }
        self.unread_token(ch);
        Ok(())
    }
    fn skip_comments(&mut self, c: Byte) -> Result<()> {
        let mut comment = String::from(char::from(c));
        comment.push_str(self.read_to_token("(),:;")?.as_str());
        while self.last_deliminator == b' ' {
            comment.push_str(self.read_to_token("(),:;")?.as_str());
        };
        if let Ok(annotation) = AnnotationParser::parse_annotation(comment.as_str()) {
            self.last_annotation = Some(annotation);
            Ok(())
        } else {
            Err(IoError::OTHER)
        }
    }

    fn skip_until(&mut self, c: Byte) -> Result<Byte> {
        let mut ch: Byte = self.read_token()?;
        while ch != c {
            ch = self.read_token()?;
        }
        Ok(ch)
    }

    fn annotation_node(&mut self, nodeRef: TreeIndex) {
        if self.last_annotation.is_some() {
            let annotation_map = self.last_annotation.take().unwrap();
            for (key, value) in annotation_map.into_iter() {
                self.get_tree().annotate_node(nodeRef, key, value);
            }
        }
    }

    fn get_tree(&mut self) -> &mut MutableTree {
        self.tree.as_mut().unwrap()
    }
    fn has_tree(&mut self) -> bool {
       match self.skip_until(b'('){
            Ok(_Byte)=>{
                self.unread_token(b'(');
                true
            },
           Err(IoError::EOF)=>false,
           Err(e)=>panic!("parsing error: {}",e)
       }

    }
}

impl<R: std::io::Read> Iterator for NewickImporter<R> {
    type Item = MutableTree;
    fn next(&mut self) -> Option<Self::Item> {
        if self.has_tree() {
         let tree = self.read_next_tree();
            match tree {
                Ok(node) => Some(node),
                Err(e) => panic!("parsing error {}", e),
            }
        } else {
            None
        }
    }
}

