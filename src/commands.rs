
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

mod collapse{
    use rebl::tree::mutable_tree::{MutableTree, TreeIndex};
    use std::collections::HashSet;
    use rand::Rng;
    use rebl::io::parser::newick_parser::AnnotationValue;


    pub fn collapse_uniform_clades(tree:&MutableTree,key:&String) {
        // Get clades that are all the same
        let mut types: Vec<HashSet<&String>> = Vec::with_capacity(tree.get_node_count());
        for node_ref in tree.preorder_iter().rev() {
            let mut set = HashSet::new();
            if tree.get_num_children(node_ref) == 0 {
                if let Some(annotation) = tree.get_annotation(node_ref, key) {
                    match annotation {
                        AnnotationValue::Discrete(s) => {
                            set.insert(s);
                            types[node_ref] = set;
                        },
                        _ => { panic!("not a discrete trait") }
                    }
                } else {
                    //TODO ignore missing
                }
            } else {
                let mut i = 0;
                while i < tree.get_num_children(node_ref) {
                    if let Some(child) = tree.get_child(node_ref, i) {
                        set = set.union(&types[child]).cloned().collect();
                        i += 1;
                    }
                }
                types[node_ref] = set;
            }
        }
    }


    fn pick_random_tip(tree: &MutableTree, node: TreeIndex)->TreeIndex {
        let kids = tree.get_num_children(node);
        if kids==0{
            return node;
        }
        let next_kid = rand::thread_rng().gen_range(0..kids);
        return pick_random_tip(tree,tree.get_child(node, next_kid).expect("child out of range"))
    }
}

pub(crate) mod annotate{
    use rebl::tree::mutable_tree::MutableTree;
    use std::collections::HashMap;
    use rebl::io::parser::newick_parser::AnnotationValue;
    use crate::Common;
    use std::path;
    use std::error::Error;


    pub fn parse_csv(traits:Option <path::PathBuf>)->Result<>{

    }

    pub fn run(common:Common,traits:Option <path::PathBuf>)->Result<(),Box<dyn Error>>{
        Ok(())
    }
    pub fn annotate_tips(mut tree:MutableTree, annotation_map:HashMap<String,HashMap<String,AnnotationValue>>){
        for taxon in annotation_map.keys(){
            let node_ref = tree.get_taxon_node(taxon).expect(&*("Taxon ".to_owned() + taxon + " not found in tree"));
            if let Some(annotations)=annotation_map.get(taxon){
                for (key,value) in annotations{
                    tree.annotate_node(node_ref, key.clone(), value.clone())
                }
            }
        }
    }
}

mod split{
    
}
pub(crate) mod stats{
    use structopt::StructOpt;
    use crate::{Common};
    use super::command_io;
    use std::io::{self, Write};
    use std::error::Error;

    #[derive(Debug, StructOpt)]
    pub enum StatsSubCommands {
        Tips,
    }
    pub fn run(common:Common, cmd:Option<StatsSubCommands>)->Result<(),Box<dyn Error>>{
        let stdout = std::io::stdout(); // get the global stdout entity
        let mut handle = stdout.lock(); // acquire a lock on it

        match cmd {
            Some(StatsSubCommands::Tips) =>{
                // info!("{:?}",common);
                let trees = command_io::parse_tree_input(common.infile).expect("error reading file");
                for tree in trees.iter(){
                    let mut i =0;
                    while i<tree.get_external_node_count(){
                        if let Some(tip)=tree.get_external_node(i){
                            if let Some(taxa)= &tip.taxon{
                                writeln!(handle, "{}", taxa)?;
                            }
                        }
                        i+=1;
                    }
                }
            Ok(())
            },
            None =>{
                let mut trees = command_io::parse_tree_input(common.infile).expect("error reading file");
                writeln!(handle,"nodes\tinternal\ttips\trootHeight\tsumbl\tmeanbl")?;

                for tree in trees.iter_mut(){
                    let root= tree.get_root().unwrap();
                    let root_height = tree.get_height(root);
                    let  nodes =tree.get_node_count();
                    let  internal=tree.get_internal_node_count();
                    let  tips =tree.get_external_node_count();
                    let mut bl = Vec::with_capacity(tree.get_node_count());
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

        }
    }
}

mod command_io {
    use std::{path, io};
    use rebl::tree::mutable_tree::MutableTree;
    use std::fs::File;
    use std::io::{BufReader, BufRead, stdout, Write, BufWriter};
    use rebl::io::parser::newick_parser::NewickParser;
    use std::path::Path;

    pub fn parse_tree_input(input: Option<path::PathBuf>) -> Result<Vec<MutableTree>,io::Error> {
        let mut trees = vec![];
        match input {
            Some(path) => {
                let file = File::open(path)?;
                let reader = BufReader::new(file);
                for line in reader.lines() {
                    if let Ok(tree)=NewickParser::parse_tree(&*line?){
                        trees.push(MutableTree::from_fixed_node(tree));
                    }
                    else{
                       warn!("no tree at this line");
                    }
                }
            }
            None => {
                info!("no file reading from stdin");
                let stdin = io::stdin();
                let  handel = stdin.lock();
                for line in handel.lines() {
                    if let Ok(tree)=NewickParser::parse_tree(&*line?){
                        trees.push(MutableTree::from_fixed_node(tree));
                    }
                }
            }
        }
        return Ok(trees);
    }
}