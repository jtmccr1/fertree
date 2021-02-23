// Needed by pest
use crate::tree::AnnotationValue;
use std::collections::HashMap;
use crate::io::Error::IoError;
use crate::tree::mutable_tree::{MutableTree, TreeIndex};
use std::str::Chars;

pub struct NewickParser<'a> {
    last_token:Option<char>,
    tree:MutableTree,
    reader:Chars<'a>,
    last_deliminator:char
}

type Result<T> = std::result::Result<T, IoError>;

/*
This is model after the newick importer in BEAST
TODO fill in the rest
 */
impl  NewickParser<'_> {
    pub fn parse_tree(str:&'static str)->Result<MutableTree>{
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
            reader: str.chars(),
           last_deliminator:'\0'
        };


        parser.skip_until('(')?;
        parser.unread_token('(');

        let root = parser.read_internal_node()?;
        //TODO hide node/node ref api
        parser.tree.set_root(Some(root));

        trace!(
            "Tree parsed in {} milli seconds ",
            start.elapsed().as_millis()
        );
        Ok(parser.tree)
    }

    fn read_internal_node(&mut self)->Result<TreeIndex>{
        let token = self.read_token()?;
        //assert =='('
        let mut children = vec![];
        children.push(self.read_branch()?);

        // read subsequent children
        while self.last_deliminator == ',' {
            children.push(self.read_branch()?);
        }

        // should have had a closing ')'
        if  self.last_deliminator != ')' {
            // throw new BadFormatException("Missing closing ')' in tree");
            Err(IoError)
        }else{
            //TODO read label here

            Ok(self.tree.make_internal_node(children))
        }
    }
    fn read_external_node(&mut self)->Result<TreeIndex>{
        let label= self.read_to_token(":();")?;

        Ok(self.tree.make_external_node(label.as_str(),None).expect("Failed to make tip"))
    }
    fn read_branch(&mut self)->Result<TreeIndex>{

        let mut length = 0.0;

        let branch= if self.next_token()? == '(' {
            // is an internal node
             self.read_internal_node()?

        } else {
            // is an external node
             self.read_external_node()?
        };
        //TODO branch comments?

        if self.last_deliminator ==':' {
            length = self.read_double(",():;")?;
        }

        self.tree.set_length(branch, length);

        Ok(branch)

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

    fn read_to_token(&mut self,deliminator:&str)->Result<String>{
        let mut space=0;
        let mut ch= '\0';
        let mut ch2='\0';
        let mut quote_char='\0';

        let mut done =false;
        let mut first =true;
        let mut quoted = false;

        self.next_token()?;
        let mut token = String::new();
        while !done {
            ch = self.read()?;
            let is_space = ch.is_whitespace();
            if quoted && ch==quote_char{
                ch2=self.read()?;
                if ch==ch2{
                    token.push(ch);
                }else{
                    self.last_deliminator=' ';
                    self.unread_token(ch2);
                    done=true;
                    quoted=false;
                }
            }else if first && (ch=='\'' || ch=='"'){
                quoted=true;
                quote_char=ch;
                first=false;
                space=0;
            }else if ch=='['{
                self.skip_comments(ch);
                self.last_deliminator=' ';
                done=true
            }else{
                if quoted{
                    if is_space{
                        space+=1;
                        ch=' ';
                    }else{
                        space=0;
                    }
                    if space<2{
                        token.push(ch);
                    }
                }else if is_space{
                    self.last_deliminator=' ';
                    done=true;
                }else if deliminator.contains(ch){
                    done=true;
                    self.last_deliminator=ch;
                }else{
                    token.push(ch);
                    first=false;
                }
            }
        }

        if self.last_deliminator.is_whitespace(){
            ch = self.next_token()?;
            while ch.is_whitespace(){
                self.read();
                ch=self.next_token()?;
            }
            if !deliminator.contains(ch){
                self.last_deliminator=self.read_token()?;
            }
        }

        Ok(token)



    }

    fn read_double(&mut self,deliminator:&str)->Result<f64>{
        let s = self.read_to_token(deliminator)?;
        //TODO capture this error

        match s.parse(){
            Ok(l)=>Ok(l),
            Err(e)=>Err(IoError)
        }
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

    fn get_last_deliminator(&self) ->char{
        self.last_deliminator
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
