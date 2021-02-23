// Needed by pest
use crate::tree::fixed_tree::FixedNode;
use crate::tree::AnnotationValue;
use std::collections::HashMap;
use crate::io::Error::IoError;
use crate::tree::mutable_tree::{MutableTree, MutableTreeNode, TreeIndex};
use std::str::Chars;

pub struct NewickParser<'a> {
    last_token:Option<char>,
    tree:MutableTree,
    reader:Chars<'a>
}

type Result<T> = std::result::Result<T, IoError>;

/*
This is model after the newick importer in BEAST
TODO fill in the rest
 */
impl  NewickParser<'_> {
    fn parse_tree(str:&'static str)->Result<MutableTree>{
        let start = std::time::Instant::now();
       let mut parser = NewickParser{
            last_token: None,
            //TODO better cleaner api through new.
            tree: MutableTree {
                nodes: vec![],
                external_nodes: vec![],
                internal_nodes: vec![],
                annotation_type: Default::default(),
                taxon_node_map: Default::default(),
                root: None,
                heights_known: false,
                branchlengths_known: false
            },
            reader: str.chars()
        };


        parser.skip_until('(')?;
        parser.unread_token('(');

        let root = parser.read_internal_node();
        //TODO hide node/node ref api
        parser.tree.set_root(Some(root.number));

        trace!(
            "Tree parsed in {} milli seconds ",
            start.elapsed().as_millis()
        );
        Ok(parser.tree)
    }
    
    fn read_internal_node(&mut self)->TreeIndex{
        let token = self.read_token()?;
        //assert =='('
        let mut children = vec![];
        children.push(self.read_branch());

        unimplemented!()
    }
    fn read_external_node(&mut self)->TreeIndex{
    unimplemented!()
    }
    fn read_branch(&mut self)->TreeIndex{
        unimplemented!()
    }
    fn unread_token(&mut self,c:char){
        self.last_token = Some(c);
    }
    fn next_token(&mut self)->Result<char>{
        match self.last_token{
            None=>{
                let c = self.read_token()?;
                self.last_token=Some(c);
                Ok(c)
            },
            Some(c)=>{
                Ok(c)
            }
        }

    }
    fn read_token(&mut self)->Result<char>{
        self.skip_space();
        let mut ch = self.read()?;
        // while hasComments && (ch == startComment || ch == lineComment) {
        while ch == '[' {
            self.skip_comments(ch);
            self.skip_space();
            ch = self.read()?;
        }

        Ok(ch)
    }
    
    fn next(&mut self)->Result<char>{
        match self.last_token{
            None=>{
                let c = self.read()?;
                self.last_token=Some(c);
                Ok(c)
            },
            Some(c)=>{
                Ok(c)
            }
        }
    }
    fn read(& mut self)->Result<char>{
        match self.last_token{
            None=>{
               if let Some(c) = self.reader.next(){
                   Ok(c)
               }else{
                   Err(IoError)
               }
            },
            Some(c)=>{
                self.last_token=None;
                Ok(c)
            }
        }
    }

    fn skip_space(&mut self)->Result<()>{
        let mut ch:char = self.read_token()?;
        while ch.is_whitespace(){
            ch = self.read_token()?;
        }
        self.unread_token(ch);
        Ok(())
    }
    fn skip_comments(&mut self,c:char)->Result<()>{
        unimplemented!()
        //TODO set up hashmap of annotations
    }

    fn skip_until(&mut self,c:char)->Result<char>{
            let mut ch:char = self.read_token()?;
            while ch!=c{
                ch = self.read_token()?;
            }
            Ok(ch)
    }

}


#[cfg(test)]
mod tests {
    use crate::io::parser::newick_parser::NewickParser;

    #[test]
    fn it_works() {
        let root = NewickParser::parse_tree("(a:1,b:4)l:5;").unwrap();
        assert_eq!(root.label.unwrap(), "l");
        let mut names = vec![];
        for child in root.children.iter() {
            if let Some(t) = &child.taxon {
                names.push(t)
            }
        }
        assert_eq!(names, vec!["a", "b"]);

        let mut bl = vec![];
        if let Some(l) = root.length {
            bl.push(l);
        }
        for child in root.children.iter() {
            if let Some(t) = child.length {
                bl.push(t)
            }
        }
        assert_eq!(bl, vec![5.0, 1.0, 4.0]);
    }

    #[test]
    fn scientific() {
        let root = NewickParser::parse_tree("(a:1E1,b:+2e-5)l:5e-1;").unwrap();
        let mut bl = vec![];
        if let Some(l) = root.length {
            bl.push(l);
        }
        for child in root.children.iter() {
            if let Some(t) = child.length {
                bl.push(t)
            }
        }
        assert_eq!(bl, vec![0.5, 10.0, 0.00002]);
    }

    #[test]
    fn quoted() {
       assert!(true, NewickParser::parse_tree("('234] ','here a *');").is_ok());
    }

    #[test]
    fn annotation() {
        assert!(NewickParser::parse_tree("(a[&test=ok],b:1);").is_ok());
    }

    #[test]
    fn whitespace() {
        assert!(NewickParser::parse_tree("  (a[&test=ok],b:1);\t").is_ok());
    }

    #[test]
    fn should_error() {
        let out = NewickParser::parse_tree("('234] ','here a *')");
        assert_eq!(true, out.is_err())
    }

    #[test]
    fn should_error_again() {
        let out = NewickParser::parse_tree("(a,b));");
        assert_eq!(true, out.is_err())
    }
}
