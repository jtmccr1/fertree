
mod introductions{
    use rebl::tree::fixed_tree::FixedNode;
    struct TaxaIntroductionLabel{
        taxa:String,
        introduction:usize,
        tmrca:f64
    }
    fn label_introductions(tree:FixedNode){

    }
}

mod thin{
}

pub(crate) mod collapse{
    use rebl::tree::mutable_tree::{MutableTree, TreeIndex};
    use std::collections::HashSet;
    use rebl::io::parser::newick_parser::AnnotationValue;
    use crate::Common;
    use crate::commands::command_io;
    use std::error::Error;
    use std::io::{Write};
    use rand::seq::SliceRandom;
//TODO set random seed.

    pub fn run(common: Common,key:String,value:String,min_size:usize)->Result<(),Box<dyn Error>> {
        let stdout = std::io::stdout(); // get the global stdout entity
        let mut handle = stdout.lock(); // acquire a lock on it

        let mut trees = command_io::parse_tree_input(common.infile)?;
        for tree in trees.iter_mut(){
            tree.calc_node_heights();
            let  new_tree = collapse_uniform_clades(tree, &key, &value,min_size);
            writeln!(handle, "{}", new_tree)?;

        }
        Ok(())
    }

    pub  fn collapse_uniform_clades(tree:&MutableTree,key:&String,value: &String,min_size:usize) ->MutableTree{
        let mut taxa:HashSet<String> = tree.external_nodes.iter()
            .map(|node|tree.get_taxon(*node)).map(|n|String::from(n.unwrap())).collect();

        let monophyletic_groups = get_monophyletic_groups(tree, tree.get_root().unwrap(), key, value);
        if monophyletic_groups.0{
            warn!("The whole tree is a monophyletic clade!")
        }
        let mut removed = 0;
        for group in monophyletic_groups.1.iter(){
            let mut rng = &mut rand::thread_rng();

            for node in group.choose_multiple(&mut rng,group.len()-min_size){
                let taxon = tree.get_taxon(*node).expect("This is not external node!");
                taxa.remove(taxon);
                removed +=1;
            }
        }
        info!("Removed : {} taxa", removed);
        let  new_tree = MutableTree::from_tree(tree,&taxa);
        new_tree
    }

    fn get_monophyletic_groups(tree:&MutableTree, node_ref:TreeIndex,key:&String,target_annotation:&String)->(bool,Vec<Vec<TreeIndex>>) {
        if tree.is_external(node_ref) {
            if let Some(annotation) = tree.get_annotation(node_ref, key) {
                match annotation {
                    AnnotationValue::Discrete(s) => {
                        return if s == target_annotation {
                            (true, vec![vec![node_ref]])
                        } else {
                            (false, vec![vec![]])
                        }
                    },
                    _ => { panic!("not a discrete trait") }
                }
            }
            // not ignoring empty nodes they are counted
            return (false,vec![]);
        }

        let mut child_output = vec![];
        for child in tree.get_children(node_ref).iter(){
            child_output.push(get_monophyletic_groups(tree, *child, key, &target_annotation))
        }
        let am_i_a_root = child_output.iter().map(|t|t.0).fold(true,|acc,b |acc&b);
        if am_i_a_root {
            let combined_child_tips = child_output.into_iter()
                .map(|t|t.1)
                .flatten()
                .flatten()
                .collect::<Vec<TreeIndex>>();
            return (true,vec![combined_child_tips]);
        }else{
            let child_tips = child_output.into_iter()
                .map(|t|t.1)
                .fold(vec![],|mut acc, next |{
                acc.extend(next);
                return acc
            } );
            return(false,child_tips)
        }

    }

}

pub(crate) mod annotate{
    use rebl::tree::mutable_tree::MutableTree;
    use std::collections::HashMap;
    use rebl::io::parser::newick_parser::AnnotationValue;
    use crate::Common;
    use std::path;
    use std::error::Error;
    use std::io::{ Write};
    use crate::commands::command_io::parse_csv;
    use crate::commands::command_io;
    use csv::Reader;
    use std::fs::File;


    pub fn run(common:Common,traits:path::PathBuf) ->Result<(),Box<dyn Error>>{
        let stdout = std::io::stdout(); // get the global stdout entity
        let mut handle = stdout.lock(); // acquire a lock on it

        let mut reader = parse_csv(traits)?;

        let mut trees = command_io::parse_tree_input(common.infile)?;
        for tree in trees.iter_mut(){
            annotate_tips(tree, &mut reader)?;
            writeln!(handle, "{}", tree)?;
        }

    Ok(())
    }
    pub fn annotate_tips(tree:& mut MutableTree, reader:&mut Reader<File>) ->Result<(),Box<dyn Error>>{
        //todo fix to handle taxa differently
        type Record = HashMap<String, Option<AnnotationValue>>;

        let header = reader.headers()?;
        let taxon_key = header.get(0).unwrap().to_string();

        for result in reader.deserialize() {
            let record: Record = result?;
            //See todo above
            if let Some(AnnotationValue::Discrete(taxon)) = record.get(&*taxon_key).unwrap() {
                if let Some(node_ref) = tree.get_taxon_node(&taxon) {
                    for (key, value) in record {
                        if key != taxon_key {
                            if let Some(annotation_value) = value {
                                tree.annotate_node(node_ref, key, annotation_value)
                            }
                        }
                    }
                }else{
                    warn!("Taxon {} not found in tree",taxon)
                }
            }
        }
        Ok(())
    }
}

pub mod extract {
    use crate::Common;
    use structopt::StructOpt;
    use std::error::Error;
    use crate::commands::command_io;
    use std::io::{ Write};
    use rebl::io::parser::newick_parser::AnnotationValue;


    #[derive(Debug, StructOpt)]
    pub enum SubCommands {
        Taxa,
        Annotations
    }

    pub fn run(common: Common, cmd: SubCommands) -> Result<(), Box<dyn Error>> {
        match cmd {
            SubCommands::Taxa => {
                taxa(common)
            },
            SubCommands::Annotations => {
                annotations(common)
            }
        }
    }

    fn taxa(common: Common) -> Result<(), Box<dyn Error>> {
        let stdout = std::io::stdout(); // get the global stdout entity
        let mut handle = stdout.lock(); // acquire a lock on it
        let trees = command_io::parse_tree_input(common.infile)?;
        for tree in trees.iter() {
            let mut i = 0;
            while i < tree.get_external_node_count() {
                if let Some(tip) = tree.get_external_node(i) {
                    if let Some(taxa) = &tip.taxon {
                        writeln!(handle, "{}", taxa)?;
                    }
                }
                i += 1;
            }
        }
        Ok(())
    }

    fn annotations(common: Common) -> Result<(), Box<dyn Error>> {
        let stdout = std::io::stdout(); // get the global stdout entity
        let mut handle = stdout.lock(); // acquire a lock on it
        let trees = command_io::parse_tree_input(common.infile)?;
        for tree in trees.iter() {
            let header = tree.annotation_type.keys().map(|k| k.clone()).collect::<Vec<String>>().join("\t");
            writeln!(handle, "{}\t{}","taxa", header)?;
            for node_ref in tree.external_nodes.iter() {
                let annotation_string = tree.annotation_type.keys()
                    .map(|k| annotation_value_string(tree.get_annotation(*node_ref, k)))
                    .collect::<Vec<String>>()
                    .join("\t");
                if let Some(taxa) = tree.get_taxon(*node_ref){
                    writeln!(handle, "{}\t{}",taxa , annotation_string)?;
                }else{
                    writeln!(handle, "{}\t{}","" , annotation_string)?;
                }

            }
        }
        Ok(())
    }

    fn annotation_value_string(value: Option<&AnnotationValue>) -> String {
        if let Some(annotation) = value {
            let value_string = annotation.to_string();
            format!("{}", value_string)
        } else {
            "".to_string()
        }
    }
}


mod split{
    
}
pub(crate) mod stats{
    use structopt::StructOpt;
    use crate::{Common};
    use super::command_io;
    use std::io::{ Write};
    use std::error::Error;

    #[derive(Debug, StructOpt)]
    pub enum SubCommands {
        Tips,
    }


    fn general_stats(common:Common)->Result<(),Box<dyn Error>> {
        let stdout = std::io::stdout(); // get the global stdout entity
        let mut handle = stdout.lock(); // acquire a lock on it
        let mut trees = command_io::parse_tree_input(common.infile).expect("error reading file");
        writeln!(handle,"nodes\tinternal\ttips\trootHeight\tsumbl\tmeanbl")?;

        for tree in trees.iter_mut(){
            let root= tree.get_root().unwrap();
            let root_height = tree.get_height(root);
            let  nodes =tree.get_node_count();
            let  internal=tree.get_internal_node_count();
            let  tips =tree.get_external_node_count();
            let mut bl = Vec::with_capacity(tree.get_node_count());
            bl.resize(tree.get_node_count(), 0.0);
            for node_ref in tree.preorder_iter() {
                if node_ref !=tree.get_root().expect("stats assume rooted nodes") {
                    if let Some(node) = tree.get_node(node_ref) {
                        if let Some(length) = node.length {
                            bl[node_ref] = length;
                        }
                    }
                }
            }
            let sum_bl = bl.iter().fold(0.0, |acc, x| acc + x);
            let mean_bl = sum_bl / ((tree.get_node_count()as f64)-1.0); //no branch on root
            writeln!(handle,"{}\t{}\t{}\t{}\t{}\t{}", nodes,internal,tips,root_height,sum_bl,mean_bl)?;
        }
        Ok(())
    }
    pub fn run(common:Common, cmd:Option<SubCommands>) ->Result<(),Box<dyn Error>>{
        //TODO move tree reading and output buffer handling out here and pass to commands

        match cmd {
            None =>{
              general_stats(common)
            }

            _ => {
                warn!("nothing done");
                Ok(())
            }
        }
    }
}

mod command_io {
    use std::{path, io};
    use rebl::tree::mutable_tree::MutableTree;
    use std::fs::File;
    use std::io::{BufReader, BufRead};
    use rebl::io::parser::newick_parser::{NewickParser};
    use std::error::Error;
    use csv::Reader;
    //TODO use iterator to read 1 line at a time
    pub fn parse_tree_input(input: Option<path::PathBuf>) -> Result<Vec<MutableTree>, io::Error> {
        let mut trees = vec![];
        match input {
            Some(path) => {
                let file = File::open(path)?;
                let reader = BufReader::new(file);
                for line in reader.lines() {
                    if let Ok(tree) = NewickParser::parse_tree(&*line?) {
                        trees.push(MutableTree::from_fixed_node(tree));
                    } else {
                        warn!("no tree at this line");
                    }
                }
            }
            None => {
                info!("no file reading from stdin");
                let stdin = io::stdin();
                let handel = stdin.lock();
                for line in handel.lines() {
                    if let Ok(tree) = NewickParser::parse_tree(&*line?) {
                        trees.push(MutableTree::from_fixed_node(tree));
                    }
                }
            }
        }
        return Ok(trees);
    }

    pub fn write_to_output(output:Option<path::PathBuf>) -> Result<(), io::Error> {
        unimplemented!()
    }
//HashMap<String,HashMap<String,AnnotationValue>>
    pub fn parse_csv(trait_file:path::PathBuf) -> Result<Reader<File>,Box<dyn Error>> {

    let file = File::open(trait_file)?;
        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .flexible(true)
            .comment(Some(b'#'))
            .from_reader(file);

        // We nest this call in its own scope because of lifetimes.
        debug!("read with headers:{:?}", rdr.headers().unwrap());

    return Ok(rdr);

    }
}