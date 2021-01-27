mod commands;

use std::{io, path};
use std::fs::File;
use std::io::{BufRead, BufReader};
use rebl::tree::mutable_tree::{MutableTree};
use rebl::parsers::newick_parser::NewickParser;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "command line tools for processing phylogenetic trees in rust")]
enum Fertree {
    Stats {
        #[structopt(flatten)]
        common: Common,
        #[structopt(subcommand)]
        cmd: Option<StatsSubCommands>,
    },
    Introductions{
        #[structopt(flatten)]
        common: Common,
        #[structopt(short, long)]
        to:String,
    }
}

#[derive(Debug, StructOpt)]
enum StatsSubCommands {
    Tips,
}
#[derive(Debug, StructOpt)]
struct Common {
    #[structopt(short, long, parse(from_os_str), help = "input tree file")]
    infile: Option<path::PathBuf>,
    #[structopt(short, long, parse(from_os_str), help = "output tree file")]
    outfile: Option<path::PathBuf>,
    #[structopt(short, long)]
    debug: bool,
    #[structopt(short, long)]
    release: bool,
}

fn main() {
    let args = Fertree::from_args();
    println!("{:?}",args);
    match Fertree::from_args() {
        Fertree::Stats { common, cmd } => {
            match cmd {
                Some(StatsSubCommands::Tips) =>{
                    println!("{:?}",common);
                    parse_input(common.infile).expect("error reading file");
                    println!("This is us getting the tips!")
                },
                None =>{
                    let mut trees = parse_input(common.infile).expect("error reading file");
                    println!("nodes\tinternal\ttips\tsumbl");

                    for tree in trees.iter_mut(){
                        let root= tree.get_root().unwrap();
                        let root_height = tree.get_height(root);
                        let  nodes =tree.get_node_count();
                        let  internal=tree.get_internal_node_count();
                        let mut bl =0.0;
                        let  tips =tree.get_external_node_count();
                        let mut preorder = tree.iter();
                        let mut visited_node = 0;
                        while let Some(node_ref) = preorder.next(tree) {
                            if let Some(node) = tree.get_node(node_ref) {
                                if let Some(length) = node.length {
                                    bl += length;
                                }
                                visited_node +=1;
                            }
                        }
                        println!("{}\t{}\t{}\t{}\t{}", nodes,internal,tips,bl,root_height);
                    }
                }

            }
        },
        Fertree::Introductions { common, to }=>{
            let trees = parse_input(common.infile).expect("error reading file");

        }

    }
}
fn parse_input(input: Option<path::PathBuf>)  -> Result<Vec<MutableTree>,io::Error> {
    let mut trees = vec![];
    match input {
        Some(path) => {
            let file = File::open(path)?;
            let reader = BufReader::new(file);
            for line in reader.lines() {
                if let Ok(tree)=NewickParser::parse_tree(&*line?){
                    trees.push(tree);
                }
               else{
                   println!("no tree at this line");
               }
            }
        }
        None => {
            println!("no file");
            let stdin = io::stdin();
            let  handel = stdin.lock();
            for line in handel.lines() {
                if let Ok(tree)=NewickParser::parse_tree(&*line?){
                    trees.push(tree);
                }
            }
        }
    }
    return Ok(trees);
}