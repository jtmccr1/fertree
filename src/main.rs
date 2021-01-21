mod tree;

use structopt::StructOpt;
use std::{path, io};
use std::io::{BufReader, BufRead, Read};
use std::fs::File;
use crate::tree::{TreeNode, Tree};

#[derive(Debug, StructOpt)]
#[structopt(about = "command line tools for processing phylogenetic trees in rust")]
enum Fertree {
    Stats {
        #[structopt(flatten)]
        common: Common,
        #[structopt(subcommand)]
        cmd: Option<StatsSubCommands>,
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
    // let args = Fertree::from_args();
    // println!("{:?}",args);
    // match Fertree::from_args() {
    //     Fertree::Stats { common, cmd } => {
    //         match cmd {
    //             Some(StatsSubCommands::Tips) =>{
    //                 println!("{:?}",common);
    //                 parse_input(common.infile).expect("error reading file");
    //                 println!("This is us getting the tips!")
    //             },
    //             None =>{
    //                 parse_input(common.infile).expect("error reading file");
    //                 println!("This would be the number of tips and such in the file/stdin");
    //             }
    //         }
    //     }
    // }

    let mut tree = Tree::new();
    let e = tree.add_node(TreeNode::new(Some(1),None));
    let d = tree.add_node(TreeNode::new(Some(2),Some(e)));
    let c = tree.add_node(TreeNode::new(Some(5),Some(e)));
    let a = tree.add_node(TreeNode::new(Some(3),Some(d)));
    let b = tree.add_node(TreeNode::new(Some(4),Some(d)));



    tree.set_root(Some(e));

    let mut preorder = tree.iter();

    preorder = tree.iter();
    while let Some(i)= preorder.next(&tree){
        let node = tree.node_at(i).expect("node to exist at given index");
        println!("{}", node.annotation.unwrap())
        }

}

fn parse_input(input: Option<path::PathBuf>) -> io::Result<()> {
    match input {
        Some(path) => {
            let file = File::open(path)?;
            let reader = BufReader::new(file);
            // TODO parse newick trees!
            for line in reader.lines() {
                println!("{}", line?)
            }
        }
        None => {
            println!("no file");
            let stdin = io::stdin();
            let mut handel = stdin.lock();
            for line in handel.lines() {
                println!("{}", line?)
            }
        }
    }
    Ok(())
}
