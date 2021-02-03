
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
    use std::io::{self, Write};
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
            annotate_tips(tree, &mut reader);
            writeln!(handle, "{}", tree)?;
        }

    Ok(())
    }
    pub fn annotate_tips(tree:& mut MutableTree, reader:&mut Reader<File>) ->Result<(),Box<dyn Error>>{
        type Record = HashMap<String, Option<AnnotationValue>>;

        let header = reader.headers()?;
        let taxon_key = header.get(0).unwrap().to_string();

        for result in reader.deserialize() {
            let record: Record = result?;
            if let Some(AnnotationValue::Discrete(taxon)) = record.get(&*taxon_key).unwrap() {
                if let Some(node_ref) = tree.get_taxon_node(&taxon) {
                    for (key, value) in record {
                        if key != taxon_key {
                            if let Some(annotation_value) = value {
                                tree.annotate_node(node_ref, key, annotation_value)
                            }
                        }
                    }
                }
            }
        }
        Ok(())
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

    fn tips(common:Common)->Result<(),Box<dyn Error>>{
        let stdout = std::io::stdout(); // get the global stdout entity
        let mut handle = stdout.lock(); // acquire a lock on it
        let trees = command_io::parse_tree_input(common.infile)?;
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
    pub fn run(common:Common, cmd:Option<StatsSubCommands>)->Result<(),Box<dyn Error>>{
        //TODO move tree reading and output buffer handling out here and pass to commands

        match cmd {
            Some(StatsSubCommands::Tips) =>{
                tips(common)
            },
            None =>{
              general_stats(common)
            }

        }
    }
}

mod command_io {
    use std::{path, io};
    use rebl::tree::mutable_tree::MutableTree;
    use std::fs::File;
    use std::io::{BufReader, BufRead, stdout, Write, BufWriter};
    use rebl::io::parser::newick_parser::{NewickParser, AnnotationValue};
    use std::path::Path;
    use std::collections::HashMap;
    use std::error::Error;
    use csv::Reader;

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