// Needed by pest
use crate::tree::AnnotationValue;
use std::collections::HashMap;
use crate::tree::mutable_tree::{MutableTree, TreeIndex};
use std::str::Chars;
use crate::io::error::IoError;


#[derive(Debug)]
pub struct NewickParser<'a> {
    last_token:Option<char>,
    tree:MutableTree,
    reader:Chars<'a>,
    last_deliminator:char,
    last_annotation:Option<HashMap<String,AnnotationValue>>
}

type Result<T> = std::result::Result<T, IoError>;

/*
This is model after the newick importer in BEAST
TODO fill in the rest
 */
impl  NewickParser<'_> {
    pub fn parse_string(input_string:String) ->Result<MutableTree>{
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
            reader: input_string.chars(),
           last_annotation:None,
           last_deliminator:'\0'
        };

        parser.skip_until('(')?;
        parser.unread_token('(');

        let root = parser.read_internal_node()?;
        //TODO hide node/node ref api
        parser.tree.set_root(Some(root));
        parser.tree.branchlengths_known=true;

        match parser.last_deliminator{
            ')'=>Err(IoError),
            ';'=>{
                trace!(
                    "Tree parsed in {} milli seconds ",
                    start.elapsed().as_millis()
                );
                Ok(parser.tree)
            },
            _=>Err(IoError)
        }
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
            let label= self.read_to_token(",:();")?;
            let node = self.tree.make_internal_node(children);
            if !label.is_empty(){
                self.tree.label_node(node, label);
            }
            self.annotation_node(node);
            //TODO root branch length?
            Ok(node)
        }
    }
    fn read_external_node(&mut self)->Result<TreeIndex>{
        let label= self.read_to_token(",:();")?;
        let node = self.tree.make_external_node(label.as_str(),None).expect("Failed to make tip");
        self.annotation_node(node);

        Ok(node)
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
            self.annotation_node(branch);
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
        self.skip_space()?;
        let mut ch = self.read()?;
        // while hasComments && (ch == startComment || ch == lineComment) {
        while ch == '[' {
            self.skip_comments(ch)?;
            self.skip_space()?;
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
                    // self.last_deliminator=' ';
                    self.unread_token(ch2);
                    // done=true;
                    quoted=false;
                }
            }else if first && (ch=='\'' || ch=='"'){
                quoted=true;
                quote_char=ch;
                first=false;
                space=0;
            }else if ch=='['{
                self.skip_comments(ch)?;
                // self.last_deliminator=' ';
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
                self.read()?;
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
        let mut ch:char = self.read()?;
        while ch.is_whitespace(){
            ch = self.read()?;
        }
        self.unread_token(ch);
        Ok(())
    }
    fn skip_comments(&mut self,c:char)->Result<()>{
        let mut comment = String::from(c);
        comment.push_str(self.read_to_token("(),:;")?.as_str());
        while self.last_deliminator==' '{
            comment.push_str(self.read_to_token("(),:;")?.as_str());
        };
        if let Ok(annotation) = AnnotationParser::parse_annotation(comment.as_str()){
            self.last_annotation = Some(annotation);
            Ok(())
        }else{
            Err(IoError)
        }

    }

    fn skip_until(&mut self,c:char)->Result<char>{
            let mut ch:char = self.read_token()?;
            while ch!=c{
                ch = self.read_token()?;
            }
            Ok(ch)
    }

    fn annotation_node(&mut self,nodeRef:TreeIndex){
        if self.last_annotation.is_some(){
            let annotation_map = self.last_annotation.take().unwrap();
            for (key, value) in annotation_map.into_iter() {
                self.tree.annotate_node(nodeRef, key, value);
            }
        }
    }

}


#[cfg(test)]
mod tests {
    use crate::io::parser::newick_parser::NewickParser;

    #[test]
    fn general_parse() {
        let tree = NewickParser::parse_string("(a:1,b:4)l;".to_string()).unwrap();
        let root = tree.get_root().unwrap();
        let label = tree.get_label(root).unwrap();
        assert_eq!( label,"l");
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
        assert_eq!(bl, vec![ 1.0, 4.0]);
    }

    #[test]
    fn scientific() {
        let tree = NewickParser::parse_string("(a:1E1,b:2e-5)l;".to_string()).unwrap();
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
       assert!(true, NewickParser::parse_string("('234] ':1,'here a *':1);".to_string()).is_ok());
    }

    #[test]
    fn comment() {
        assert!(NewickParser::parse_string("(a[&test=ok],b:1);".to_string()).is_ok());
    }

    #[test]
    fn whitespace() {
        assert!(NewickParser::parse_string("  (a,b:1);\t".to_string()).is_ok());
    }

    #[test]
    fn should_error() {
        let out = NewickParser::parse_string("('234] ','here a *')".to_string());
        assert_eq!(true, out.is_err())
    }

    #[test]
    fn should_error_again() {
        let out = NewickParser::parse_string("(a,b));".to_string());
        assert_eq!(true, out.is_err())
    }
}
