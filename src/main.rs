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
    Stats {
        // #[structopt(flatten)]
        // common: Common,
        #[structopt(subcommand)]
        cmd: Option<stats::SubCommands>,
    },
    Introductions {
        // #[structopt(flatten)]
        // common: Common,
        #[structopt(short, long)]
        to: String,
    },
    Annotate {
        // #[structopt(flatten)]
        // common: Common,
        #[structopt(short, long, parse(from_os_str), help = "trait csv with taxa labels as first field")]
        traits: path::PathBuf,
    },
    Extract {
        // #[structopt(flatten)]
        // common: Common,
        #[structopt(subcommand)]
        cmd: extract::SubCommands,
    },
    Collapse {
        // #[structopt(flatten)]
        // common: Common,
        #[structopt(short, long, help = "annotation key we are collapsing by. must be discrete")]
        annotation: String,
        #[structopt(short, long, help = "annotation value we are collapsing by")]
        value: String,
        #[structopt(short, long, help = "the minimum clade size", default_value = "1")]
        min_size: usize,
    },
    Split {
        // #[structopt(flatten)]
        // common: Common,
        // #[structopt(short, long, help = "annotation key we are collapsing by. must be discrete")]
        // max_size: String,
        // short and long flags (-d, --debug) will be deduced from the field's name
        #[structopt(short, long, help = "Don't split tree but print the number of trees at different cut-offs")]
        explore: bool,
        #[structopt(short, long, help = "the minimum clade size",required_if("explore","true"))]
        min_size: Option<usize>,
    },
}


#[derive(Debug, StructOpt)]
pub struct Common {
    #[structopt(short, long, parse(from_os_str), help = "input tree file", global = true)]
    infile: Option<path::PathBuf>,
    #[structopt(short, long, parse(from_os_str), help = "output tree file", global = true)]
    outfile: Option<path::PathBuf>,
    #[structopt(short, long, global = true)]
    release: bool,
    //TODO implement this log file option
    #[structopt(short, long, parse(from_os_str), help = "logfile", global = true)]
    logfile: Option<path::PathBuf>,
    //TODO include verbosity flag here to overwrite env_logger
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
        Fertree::Split {min_size,explore}=>{
            split::run(tree_importer,min_size,explore)
        }

        // Fertree::Introductions { tree_importer, to }=>{
        //     Ok(())
        // },
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



