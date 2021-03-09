use std::io::{BufReader, Read};
use crate::tree::mutable_tree::{MutableTree, TreeIndex};
use std::collections::{HashMap, HashSet};
use crate::tree::AnnotationValue;
use crate::io::error::IoError;
use crate::io::parser::annotation_parser::AnnotationParser;
use serde::de::value::StrDeserializer;

type Byte = u8;
type Result<T> = std::result::Result<T, IoError>;

struct NexusImporter<R> {
    last_token: String,
    last_byte: Option<u8>,
    tree: Option<MutableTree>,
    reader: BufReader<R>,
    last_deliminator: u8,
    last_annotation: Option<HashMap<String, AnnotationValue>>,
    taxa: HashSet<String>,
    taxa_translation:HashMap<String,String>, // TODO make taxon
    reading_trees:bool
}
enum NexusBlock{
    TAXA,
    TREES,
}

impl<R: std::io::Read> NexusImporter<R> {
    pub fn from_reader(reader:R)->Self{
        NexusImporter {
            last_token: "".to_string(),
            last_byte: None,
            tree: None,
            reader: BufReader::new(reader),
            last_annotation: None,
            taxa: Default::default(),
            taxa_translation: Default::default(),
            last_deliminator: b'\0',
            reading_trees: false
        }
    }
    pub fn read_tree(input_reader: BufReader<R>) -> Result<MutableTree> {
        let mut parser = NexusImporter {
            last_token: "".to_string(),
            last_byte: None,
            tree: None,
            reader: input_reader,
            last_annotation: None,
            taxa: Default::default(),
            taxa_translation: Default::default(),
            last_deliminator: b'\0',
            reading_trees: false
        };

        parser.read_next_tree()
    }


    fn prep_for_trees(&mut self) ->Result<()>{
        loop {
            let block = self.find_next_block();
            match block {
                Ok(NexusBlock::TAXA) => self.read_taxa_block()?,
                Ok(NexusBlock::TREES) => {
                    self.read_translation_list();
                    self.reading_trees = true;
                   break
                },
                Err(IoError::EOF)=>break,
                Err(e)=>{
                    warn!("{}",e);
                }
            }
        }
        Ok(())
    }


    fn find_next_block(&mut self)->Result<NexusBlock>{
        loop {
            let token = self.read_token("")?;
            if token.eq_ignore_ascii_case("begin"){
                break;
            }
        }
        let block = self.read_token(";")?;
        if block.eq_ignore_ascii_case("taxa") {
            Ok(NexusBlock::TAXA)
        }else if block.eq_ignore_ascii_case("trees"){
            Ok(NexusBlock::TREES)
        }else{
            Err(IoError::FORMAT("unsupported nexus block ".to_string() + &*block))
        }
    }
    fn find_end_block(&mut self)->Result<()>{
        loop {
            let token = self.read_token(";")?;
            if token.eq_ignore_ascii_case("end")||token.eq_ignore_ascii_case("endblock"){
                break;
            }
        }
        Ok(())
    }
    fn read_taxa_block(&mut self)->Result<()>{
        let mut taxa_count =0;
        let token = self.read_token("")?;

        if token.eq_ignore_ascii_case("DIMENSIONS") {
            let token2 = self.read_token("=;")?;
            if !token2.eq_ignore_ascii_case("NTAX") {
               panic!("missing ntax tag".to_string())
            };
            taxa_count = self.read_int(";")?;
        };
        let taxalabels = self.read_token(";")?;
        if taxalabels.eq_ignore_ascii_case("TAXLABELS"){
            loop {
                let taxa = self.read_token(";")?.trim().to_string();
                if taxa.len()>0 {
                    let uniq = self.taxa.insert(taxa.to_string());
                    if !uniq{
                        panic!("duplicated taxa:".to_string()+ &*taxa)
                    }
                }
                if self.last_deliminator==b';' {
                    break;
                }
            };
            if taxa_count!= self.taxa.len(){
                panic!("taxa count does not match ntax tag".to_string())
            };
            Ok(())
        }else{
            panic!("Could not find taxlabels in taxa block")
        }


    }

    fn read_translation_list(&mut self)->Result<()>{
        let token = self.read_token(";")?;
        if token.eq_ignore_ascii_case("TRANSLATE") {
            loop {
                let key = self.read_token(",;")?;
                if self.last_deliminator == b',' || self.last_deliminator == b';' {
                   break Err(IoError::FORMAT("missing taxon label in translate section of trees block".to_string()));
                } else {
                    let taxon = self.read_token(",;")?;
                    //TODO build from Taxa block if needed
                    if let Some(key) = self.taxa_translation.insert(key, taxon) {
                       break Err(IoError::FORMAT("translate map uses ".to_string() + &key + "twice"))
                    }
                }
                if self.last_deliminator==b';' {
                    //now prep for reading trees that come next
                    self.read_token(";");
                    break Ok(())
                }

            }
        }else{
            Ok(())
        }

    }


    fn read_next_tree(&mut self) -> Result<MutableTree> {
        if !self.reading_trees{
            self.prep_for_trees();
        }
        if self.last_token.eq_ignore_ascii_case("UTREE") || self.last_token.eq_ignore_ascii_case("TREE") {
            let start = std::time::Instant::now();
            self.tree = Some(MutableTree::new());
            if self.last_byte == Some(b'*') {
                // Star is used to specify a default tree - ignore it
                self.read_byte();
            }

            let label = self.read_token("=;")?;
            let tree_annotation = self.last_annotation.take();

            // ignoring comment that may have been picked up
            if self.last_deliminator != b'=' {
               panic!("Missing  '=' or label for tree {}", label.as_str())
            }

            if self.next_byte()? != b'(' {
                panic!("Missing tree definition in TREE command of TREES block".to_string())
            } else {
                let rooted_comment = self.last_annotation.take();
                let root = self.read_internal_node()?;

                self.get_tree().set_root(Some(root));
                self.get_tree().branchlengths_known = true;
                self.get_tree().set_id(label);

                match self.last_deliminator {
                    b')' => Err(IoError::FORMAT("Tree parsing ended with ')'".to_string())),
                    b';' => {
                        trace!(
                            "Tree parsed in {} milli seconds ",
                            start.elapsed().as_millis()
                        );
                        if let Some(annotation) = tree_annotation {
                            for (key, value) in annotation.into_iter() {
                                self.get_tree().annotate_tree(key, value);
                            }
                        }
                        if let Some(annotation) = rooted_comment {
                            for (key, value) in annotation.into_iter() {
                                self.get_tree().annotate_tree(key, value);
                            }
                        }
                        self.read_token(";")?;
                        Ok(self.tree.take().unwrap())
                    }
                    _ => Err(IoError::FORMAT("You may need to read to check for a root branch or annotation".to_string()))
                }
            }
        } else if self.last_token.eq_ignore_ascii_case("ENDBLOCK") || self.last_token.eq_ignore_ascii_case("END") {
            Err(IoError::EOF)
        } else {
            Err(IoError::FORMAT(String::from("unknown command in tree block") + &*self.last_token))
        }
    }

    fn read_internal_node(&mut self) -> Result<TreeIndex> {
        let token = self.read_byte()?;
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
        let mut ch = b'\0';
        let mut ch2 = b'\0';
        let mut quote_char = b'\0';

        let mut done = false;
        let mut first = true;
        let mut quoted = false;

        self.next_byte()?;
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
            ch = self.next_byte()?;
            while char::from(ch).is_whitespace() {
                self.read()?;
                ch = self.next_byte()?;
            }

            if delims.contains(&ch) {
                self.last_deliminator = self.read_byte()?;
            }
        }
        self.last_token=token;
        Ok(self.last_token.clone()) //TODO is this poor form
    }

    fn read_double(&mut self, deliminator: &str) -> Result<f64> {
        let s = self.read_token(deliminator)?;
        //TODO capture this error

        match s.parse() {
            Ok(l) => Ok(l),
            Err(e) => Err(IoError::OTHER)
        }
    }
    fn read_int(&mut self, deliminator: &str) -> Result<usize> {
        let s = self.read_token(deliminator)?;
        //TODO capture this error

        match s.parse() {
            Ok(l) => Ok(l),
            Err(e) => Err(IoError::OTHER)
        }
    }

    fn read(&mut self) -> Result<Byte> {
        let mut buf: [u8; 1] = [0; 1];
        match self.last_byte {
            None => {
                match self.reader.read(&mut buf) {
                    Ok(1) => Ok(buf[0]),
                    Ok(0) => Err(IoError::EOF),
                    _ => Err(IoError::OTHER)
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
        let mut comment_depth =1;
        while comment_depth>0{
            let ch = self.read()?;
            if ch==b'['{
                comment_depth+=1;
            }else if ch==b']'{
                comment_depth-=1;
            }
            comment.push(char::from(ch));
        }
        debug!("Comment: {}",comment);
        if let Ok(annotation) = AnnotationParser::parse_annotation(comment.as_str()) {
            self.last_annotation = Some(annotation);
            Ok(())
        } else {
            Err(IoError::OTHER)
        }
    }

    fn skip_until(&mut self, c: Byte) -> Result<Byte> {
        let mut ch: Byte = self.read_byte()?;
        while ch != c {
            ch = self.read_byte()?;
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
}

impl<R: std::io::Read> Iterator for NexusImporter<R> {
    type Item = MutableTree;
    fn next(&mut self) -> Option<Self::Item> {
            if !self.reading_trees{
                self.prep_for_trees();
            };
            match self.reading_trees {
                false=>None,
                true =>{
                    match  self.read_next_tree() {
                        Ok(node) => Some(node),
                        Err(IoError::EOF)=>None,
                        Err(e) => panic!("parsing error {}", e),
                    }
                }
            }

    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn test() {
    let nexus ="#NEXUS
        BEGIN TAXA;
        DIMENSIONS NTAX=4;
        TAXLABELS Tip0 Tip1 Tip2 Tip3;
        END;
        BEGIN TREES;
        TREE tree0 = (Tip0:0.10948830688813957,Tip1:0.08499321350697361,(Tip2:0.17974073029346938,Tip3:0.1702835785780057):0.19361872371858507);
        END;";
        let tree = NexusImporter::read_tree(BufReader::new(nexus.as_bytes()));
        assert!(tree.is_ok())
    }

    #[test]
    fn iterator(){
        let nexus ="#NEXUS
        BEGIN TAXA;
        DIMENSIONS NTAX=4;
        TAXLABELS Tip0 Tip1 Tip2 Tip3;
        END;
        BEGIN TREES;
        TREE tree0 = (Tip0:0.10948830688813957,Tip1:0.08499321350697361,(Tip2:0.17974073029346938,Tip3:0.1702835785780057):0.19361872371858507);
        TREE tree1 = (Tip0:0.10948830688813957,Tip1:0.08499321350697361,(Tip2:0.17974073029346938,Tip3:0.1702835785780057):0.19361872371858507);
        END;";

        let trees = NexusImporter::from_reader(nexus.as_bytes());
        let mut tree_ids = vec![];
        for mut tree in trees{
            tree_ids.push(tree.id.take().unwrap());
        }
        assert_eq!(vec!["tree0","tree1"],tree_ids)
    }
    #[test]
    fn traslation(){
        let nexus ="#NEXUS
        BEGIN TAXA;
        DIMENSIONS NTAX=4;
        TAXLABELS Tip0 Tip1 Tip2 Tip3;
        END;
        BEGIN TREES;
        translate
        0 Tip0,
        1 Tip1,
        2 Tip2,
        3 Tip3
        ;
        TREE tree0 = (0:0.10948830688813957,1:0.08499321350697361,(2:0.17974073029346938,3:0.1702835785780057):0.19361872371858507);
        TREE tree1 = (0:0.10948830688813957,1:0.08499321350697361,(2:0.17974073029346938,3:0.1702835785780057):0.19361872371858507);
        END;";

        let trees = NexusImporter::from_reader(nexus.as_bytes());
        let mut tree_ids = vec![];
        for mut tree in trees{
            tree_ids.push(tree.id.take().unwrap());
        }
        assert_eq!(vec!["tree0","tree1"],tree_ids)
    }

    #[test]
    fn annotation(){
        let nexus ="#NEXUS
        BEGIN TAXA;
        DIMENSIONS NTAX=4;
        TAXLABELS Tip0 Tip1 Tip2 Tip3;
        END;
        BEGIN TREES;
        translate
        0 Tip0,
        1 Tip1,
        2 Tip2,
        3 Tip3
        ;
        TREE tree0 = (0:0.10948830688813957,1:0.08499321350697361,(2:0.17974073029346938,3:0.1702835785780057):0.19361872371858507)[&location=\"UK\"];
        END;";

        let trees = NexusImporter::from_reader(nexus.as_bytes());
        let mut correct = false;
        for mut tree in trees{
            let root =tree.get_root().unwrap();
           if let Some(AnnotationValue::Discrete(location)) = tree.get_annotation(root, "location"){
               if location =="UK"{
                   correct =true;
               }
           }

        }
        assert!(correct);
    }
    #[test]
    fn with_tree_comments(){
        let nexus ="#NEXUS
        BEGIN TAXA;
        DIMENSIONS NTAX=4;
        TAXLABELS Tip0 Tip1 Tip2 Tip3;
        END;
        BEGIN TREES;
        translate
        0 Tip0,
        1 Tip1,
        2 Tip2,
        3 Tip3
        ;
        TREE tree0 [&joint=1.9] = [&R] (0:0.10948830688813957,1:0.08499321350697361,(2:0.17974073029346938,3:0.1702835785780057):0.19361872371858507)[&location=\"UK\"];
        END;";

        let trees = NexusImporter::from_reader(nexus.as_bytes());
        let mut correct = false;
        for mut tree in trees{
            let root =tree.get_root().unwrap();
            if let Some(AnnotationValue::Continuous(num)) = tree.get_tree_annnotation("joint"){
                if num < &2.0 {
                    correct =true;
                }
            }
        }
        assert!(correct);
    }

}