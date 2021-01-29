mod commands;

use std::{io, path};
use std::fs::File;
use std::io::{BufRead, BufReader};
use rebl::tree::mutable_tree::{MutableTree};
use structopt::StructOpt;
use rebl::io::parser::newick_parser::NewickParser;
use commands::stats;

#[derive(Debug, StructOpt)]
#[structopt(about = "command line tools for processing phylogenetic trees in rust")]
enum Fertree {
    Stats {
        #[structopt(flatten)]
        common: Common,
        #[structopt(subcommand)]
        cmd: Option<stats::StatsSubCommands>,
    },
    Introductions{
        #[structopt(flatten)]
        common: Common,
        #[structopt(short, long)]
        to:String,
    }
}


#[derive(Debug, StructOpt)]
pub struct Common {
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
            stats::run(common, cmd);

        },
        Fertree::Introductions { common, to }=>{

        }

    }
}
