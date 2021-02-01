mod commands;

use rebl::tree::mutable_tree::{MutableTree};
use structopt::StructOpt;
use rebl::io::parser::newick_parser::NewickParser;
use commands::{stats,annotate};
use std::path;

#[macro_use]
extern crate log;

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
    },
    Annotate{
        #[structopt(flatten)]
        common: Common,
        #[structopt(short, long, parse(from_os_str), help = "trait csv with taxa labels as frist field")]
        traits: Option <path::PathBuf>,
    },
    ExtractAnnotations {
        #[structopt(flatten)]
        common: Common,
    }

}


#[derive(Debug, StructOpt)]
pub struct Common {
    #[structopt(short, long, parse(from_os_str), help = "input tree file",global=true)]
    infile: Option<path::PathBuf>,
    #[structopt(short, long, parse(from_os_str), help = "output tree file",global=true)]
    outfile: Option<path::PathBuf>,
    #[structopt(short, long,global=true)]
    debug: bool,
    #[structopt(short, long,global=true)]
    release: bool,
}

fn main() {
    env_logger::init();
    info!("starting up");
    let args = Fertree::from_args();
    debug!("{:?}",args);
    let start = std::time::Instant::now();
   let result =  match Fertree::from_args() {
        Fertree::Stats { common, cmd } => {
            stats::run(common, cmd)
        },
       Fertree::Annotate{common,traits}=>{
           annotate::run(common,traits)
       }
        Fertree::Introductions { common, to }=>{
            Ok(())
        },
       (_)=>{
           warn!("not implemented");
           Ok(())
       }

    };
    info!("{} seconds elapsed",start.elapsed().as_secs());
    match result{
        Ok(_) => {
            std::process::exit(exitcode::OK);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(exitcode::IOERR);
        }
    }
}
