use crate::tree::mutable_tree::{MutableTree, TreeIndex};
use std::io::{Read, BufReader};
use crate::io::error::{IoError};
use std::collections::HashMap;
use crate::tree::AnnotationValue;
use crate::io::parser::annotation_parser::AnnotationParser;
use crate::io::parser::tree_importer::TreeImporter;

type Result<T> = std::result::Result<T, IoError>;
type Byte = u8;

pub struct NewickImporter<R> {
    last_byte: Option<u8>,
    tree: Option<MutableTree>,
    reader: BufReader<R>,
    last_deliminator: u8,
    last_annotation: Option<HashMap<String, AnnotationValue>>,
}

impl<R: std::io::Read> NewickImporter<R> {
    pub fn from_reader(reader: R) -> Self {
        NewickImporter {
            last_byte: None,
            //TODO better cleaner api through new.
            tree: None,
            reader: BufReader::new(reader),
            last_annotation: None,
            last_deliminator: b'\0',
        }
    }
    pub fn read_tree(input_reader: BufReader<R>) -> Result<MutableTree> {
        let mut parser = NewickImporter {
            last_byte: None,
            //TODO better cleaner api through new.
            tree: None,
            reader: input_reader,
            last_annotation: None,
            last_deliminator: b'\0',
        };

        parser.read_next_tree()
    }
    fn read_internal_node(&mut self) -> Result<TreeIndex> {
        self.read_byte()?;
        //assert =='('
        let mut children = vec![self.read_branch()?];

        // read subsequent children
        while self.last_deliminator == b',' {
            children.push(self.read_branch()?);
        }

        // should have had a closing ')'
        if self.last_deliminator != b')' {
            // throw new BadFormatException("Missing closing ')' in tree");
            Err(IoError::Other)
        } else {
            let label = self.read_token(",:();")?;
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
        let label = self.read_token(",:();")?;
        let node = self.get_tree().make_external_node(label.as_str(), None).expect("Failed to make tip");
        self.annotation_node(node);

        Ok(node)
    }
    fn read_branch(&mut self) -> Result<TreeIndex> {
        let mut length = 0.0;

        let branch = if self.next_byte()? == b'(' {
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
    fn unread_byte(&mut self, c: Byte) {
        self.last_byte = Some(c);
    }

    fn next_byte(&mut self) -> Result<Byte> {
        match self.last_byte {
            None => {
                let c = self.read_byte()?;
                self.last_byte = Some(c);
                Ok(c)
            }
            Some(c) => {
                Ok(c)
            }
        }
    }
    fn read_byte(&mut self) -> Result<Byte> {
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

    fn read_token(&mut self, deliminator: &str) -> Result<String> {
        let delims = deliminator.bytes().collect::<Vec<Byte>>();
        let mut space = 0;
        let mut quote_char = b'\0';

        let mut done = false;
        let mut first = true;
        let mut quoted = false;

        self.next_byte()?;
        let mut token = String::new();
        while !done {
            let mut ch = self.read()?;
            let is_space = char::from(ch).is_whitespace();
            if quoted && ch == quote_char {
                let ch2 = self.read()?;
                if ch == ch2 {
                    token.push(char::from(ch));
                } else {
                    // self.last_deliminator=' ';
                    self.unread_byte(ch2);
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
                self.last_deliminator = b' ';
                done = true
            } else if quoted {
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
        if char::from(self.last_deliminator).is_whitespace() {
            let mut ch = self.next_byte()?;
            while char::from(ch).is_whitespace() {
                self.read()?;
                ch = self.next_byte()?;
            }

            if delims.contains(&ch) {
                self.last_deliminator = self.read_byte()?;
            }
        }

        Ok(token)
    }

    fn read_double(&mut self, deliminator: &str) -> Result<f64> {
        let s = self.read_token(deliminator)?;
        //TODO capture this error

        match s.parse() {
            Ok(l) => Ok(l),
            Err(e) => panic!("{}", e)
        }
    }

    fn read(&mut self) -> Result<Byte> {
        let mut buf: [u8; 1] = [0; 1];
        match self.last_byte {
            None => {
                match self.reader.read(&mut buf) {
                    Ok(1) => Ok(buf[0]),
                    Ok(0) => Err(IoError::Eof),
                    _ => Err(IoError::Other)
                }
            }
            Some(c) => {
                self.last_byte = None;
                Ok(c)
            }
        }
    }

    fn skip_space(&mut self) -> Result<()> {
        let mut ch: Byte = self.read()?;
        while char::from(ch).is_whitespace() {
            ch = self.read()?;
        }
        self.unread_byte(ch);
        Ok(())
    }
    fn skip_comments(&mut self, c: Byte) -> Result<()> {
        let mut comment = String::from(char::from(c));
        let mut comment_depth = 1;
        while comment_depth > 0 {
            let ch = self.read()?;
            if ch == b'[' {
                comment_depth += 1;
            } else if ch == b']' {
                comment_depth -= 1;
            }
            comment.push(char::from(ch));
        }
        trace!("Comment: {}", comment);
        if let Ok(annotation) = AnnotationParser::parse_annotation(comment.as_str()) {
            self.last_annotation = Some(annotation);
            Ok(())
        } else {
            panic!("Error parsing annotation")
        }
    }

    fn skip_until(&mut self, c: Byte) -> Result<Byte> {
        let mut ch: Byte = self.read_byte()?;
        while ch != c {
            ch = self.read_byte()?;
        }
        Ok(ch)
    }

    fn annotation_node(&mut self, node_ref: TreeIndex) {
        if self.last_annotation.is_some() {
            let annotation_map = self.last_annotation.take().unwrap();
            for (key, value) in annotation_map.into_iter() {
                self.get_tree().annotate_node(node_ref, key, value);
            }
        }
    }

    fn get_tree(&mut self) -> &mut MutableTree {
        self.tree.as_mut().unwrap()
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

impl<R: std::io::Read> TreeImporter<R> for NewickImporter<R> {
    fn has_tree(&mut self) -> bool {
        match self.skip_until(b'(') {
            Ok(_byte) => {
                self.unread_byte(b'(');
                true
            }
            Err(IoError::Eof) => false,
            Err(e) => panic!("parsing error: {}", e)
        }
    }
    fn read_next_tree(&mut self) -> Result<MutableTree> {
        let start = std::time::Instant::now();
        self.tree = Some(MutableTree::new());
        self.skip_until(b'(')?;
        self.unread_byte(b'(');

        let root = self.read_internal_node()?;
        //TODO hide node/node ref api
        self.get_tree().set_root(Some(root));
        self.get_tree().branchlengths_known = true;

        match self.last_deliminator {
            b')' => Err(IoError::Other),
            b';' => {
                trace!(
                    "Tree parsed in {} milli seconds ",
                    start.elapsed().as_millis()
                );
                Ok(self.tree.take().unwrap())
            }
            _ => Err(IoError::Other)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn general_parse() {
        let tree = NewickImporter::read_tree(BufReader::new("(a:1,b:4)l;".as_bytes())).unwrap();
        let root = tree.get_root().unwrap();
        let label = tree.get_label(root).unwrap();
        assert_eq!(label, "l");
        let mut names = vec![];
        for child in tree.get_children(root).iter() {
            if let Some(t) = tree.get_taxon(*child) {
                names.push(t)
            }
        }
        assert_eq!(names, vec!["a", "b"]);

        let mut bl = vec![];
        if let Some(l) = tree.get_length(root) {
            bl.push(l);
        }
        for child in tree.get_children(root).iter() {
            if let Some(t) = tree.get_length(*child) {
                bl.push(t)
            }
        }
        assert_eq!(bl, vec![1.0, 4.0]);
    }

    #[test]
    fn scientific() {
        let tree = NewickImporter::read_tree(BufReader::new("(a:1E1,b:2e-5)l;".as_bytes())).unwrap();
        let root = tree.get_root().unwrap();
        let mut bl = vec![];
        if let Some(l) = tree.get_length(root) {
            bl.push(l);
        }
        for child in tree.get_children(root).iter() {
            if let Some(t) = tree.get_length(*child) {
                bl.push(t)
            }
        }
        assert_eq!(bl, vec![10.0, 0.00002]);
    }

    #[test]
    fn quoted() {
        assert!(true, "{}", NewickImporter::read_tree(BufReader::new("('234] ':1,'here a *':1);".as_bytes())).is_ok());
    }

    #[test]
    fn comment() {
        assert!(NewickImporter::read_tree(BufReader::new("(a[&test=ok],b:1);".as_bytes())).is_ok());
    }

    #[test]
    fn double_comment() {
        assert!(NewickImporter::read_tree(BufReader::new("(a[&test=ok,value=0.9],b:1);".as_bytes())).is_ok());
    }

    #[test]
    fn whitespace() {
        assert!(NewickImporter::read_tree(BufReader::new("  (a,b:1);\t".as_bytes())).is_ok());
    }

    #[test]
    fn should_error() {
        let out = NewickImporter::read_tree(BufReader::new("('234] ','here a *')".as_bytes()));
        assert_eq!(true, out.is_err())
    }

    #[test]
    fn should_error_again() {
        let out = NewickImporter::read_tree(BufReader::new("(a,b));".as_bytes()));
        assert_eq!(true, out.is_err())
    }
}