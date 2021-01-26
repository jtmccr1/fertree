mod commands;

use std::{io, path};
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use rebl::tree::mutable_tree::{MutableTree, MutableTreeNode};
use rebl::parsers::newick_parser::NewickParser;
use structopt::StructOpt;
use rebl::tree::fixed_tree::FixedNode;

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
                    let trees = parse_input(common.infile).expect("error reading file");
                    println!("nodes\tinternal\ttips\tsumbl");

                    for tree in trees.iter(){
                        let mut nodes =tree.nodes.len();
                        let mut internal=tree.internal_nodes.len();
                        let mut bl =0.0;
                        let mut tips =tree.external_nodes.len();
                        let mut preorder = tree.iter();

                        while let Some(nodeRef) = preorder.next(tree) {
                            println!("{}",nodeRef);
                            if let Some(node) = tree.get_node(nodeRef) {
                                if let Some(length) = node.length {
                                    bl += length;
                                }
                            }
                        }
                        println!("{:?}", tree.get_node(2));
                        println!("{}\t{}\t{}\t{}", nodes,internal,tips,bl)
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
            let mut handel = stdin.lock();
            for line in handel.lines() {
                if let Ok(tree)=NewickParser::parse_tree(&*line?){
                    trees.push(tree);
                }
            }
        }
    }
    return Ok(trees);
}