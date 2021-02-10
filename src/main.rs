mod commands;

use structopt::StructOpt;
use commands::{stats, annotate, extract, collapse};
use rebl::io::parser::newick_importer;
use std::{path, io};
use crate::commands::split;
#[macro_use]
extern crate log;


#[derive(Debug, StructOpt)]
#[structopt(about = "command line tools for processing phylogenetic trees in rust", rename_all = "kebab-case")]
struct Cli {
    #[structopt(flatten)]
    common: Common,
    #[structopt(subcommand)]
    cmd: Fertree,
}

#[derive(Debug, StructOpt)]
enum Fertree {
    /// A few useful stats about the trees
    Stats {
        #[structopt(subcommand)]
        cmd: Option<stats::SubCommands>,
    },
    /// Label annotation state changes along a tree
    Introductions {
        #[structopt(short, long)]
        to: String,
    },
    /// Annotate the tips of a tree from a tsv file.
    Annotate {
        #[structopt(short, long, parse(from_os_str), help = "trait tsv with taxa labels as first field")]
        traits: path::PathBuf,
    },
    /// Extract data from a tree
    Extract {
        #[structopt(subcommand)]
        cmd: extract::SubCommands,
    },
    /// Collapse (i.e. subsample) monophyletic clades into a set number of tips
    Collapse {
        #[structopt(short, long, help = "annotation key we are collapsing by. must be discrete")]
        annotation: String,
        #[structopt(short, long, help = "annotation value we are collapsing by")]
        value: String,
        #[structopt(short, long, help = "the minimum clade size", default_value = "1")]
        min_size: usize,
    },
    /// Split an input tree into subtrees of set sizes.
    ///
    /// The --explore tag by itself gives an overview of how many trees there
    /// are at different cutoffss. When combined with a min_size, --explore
    /// outputs the number of tips in each tree.
    Split {

        #[structopt(short, long, help = "Don't split tree but print the number of trees at different cut-offs")]
        explore: bool,
        #[structopt(short, long, help = "relax the minimum clade size so that the root subtree is a separate subtree.")]
        relaxed: bool,
        #[structopt(short, long, help = "the minimum clade size",required_if("explore","true"))]
        min_size: Option<usize>,
    },
    //resolve polytomies in a variety of ways
    Resolve{
        #[structopt(subcommand)]
        cmd: resolve::SubCommands,
    }
}


#[derive(Debug, StructOpt)]
pub struct Common {
    #[structopt(short, long, parse(from_os_str), help = "input tree file", global = true)]
    infile: Option<path::PathBuf>,
    // #[structopt(short, long, parse(from_os_str), help = "output tree file", global = true)]
    // outfile: Option<path::PathBuf>,
    // //TODO implement this log file option
    // #[structopt(short, long, parse(from_os_str), help = "logfile", global = true)]
    // logfile: Option<path::PathBuf>,
}

fn main() {
    //TODO change env variable
    env_logger::init();
    trace!("starting up");
    let args = Cli::from_args();
    debug!("{:?}", args);
    let start = std::time::Instant::now();
    let stdin = io::stdin();
    let tree_importer = match args.common.infile {
        Some(path) => newick_importer::NewickImporter::from_path(path).expect("Error reading file"),
        None => {
            newick_importer::NewickImporter::from_console(&stdin)
        }
    };

    let result = match args.cmd {
        Fertree::Stats { cmd } => {
            stats::run(tree_importer, cmd)
        }
        Fertree::Annotate { traits } => {
            annotate::run(tree_importer, traits)
        }
        Fertree::Extract { cmd } => {
            extract::run(tree_importer, cmd)
        }
        Fertree::Collapse { annotation, value, min_size } => {
            collapse::run(tree_importer, annotation, value, min_size)
        },
        Fertree::Split {min_size,explore,relaxed}=>{
            split::run(tree_importer,min_size,explore,!relaxed)
        }
        _ => {
            warn!("not implemented");
            Ok(())
        }
    };
    info!("{} seconds elapsed", start.elapsed().as_secs());
    match result {
        Ok(_) => {
            std::process::exit(exitcode::OK);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(exitcode::IOERR);
        }
    }

}



