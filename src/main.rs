mod commands;

use crate::commands::{split, transmission_lineage};
use commands::{annotate, clades, extract, resolve, stats};
use std::{io, path};
use structopt::StructOpt;
use std::fs::File;
use rebl::io::parser::tree_importer::TreeImporter;
use std::io::{StdinLock};
use std::error::Error;
use rebl::io::parser::{nexus_importer, newick_importer};
use rebl::io::parser::nexus_importer::NexusImporter;

#[macro_use]
extern crate log;

#[derive(Debug, StructOpt)]
#[structopt(
about = "command line tools for processing phylogenetic trees in rust",
rename_all = "kebab-case"
)]
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
    /// Annotate the tips of a tree from a tsv file.
    Annotate {
        #[structopt(
        short,
        long,
        parse(from_os_str),
        help = "trait tsv with taxa labels as first field"
        )]
        traits: path::PathBuf,
    },
    /// Extract data from a tree
    Extract {
        #[structopt(subcommand)]
        cmd: extract::SubCommands,
    },
    /// Collapse (i.e. subsample) monophyletic clades into a set number of tips
    Clades {
        #[structopt(subcommand)]
        cmd:clades::SubCommands,
    },
    /// Split an input tree into subtrees of set sizes.
    ///
    /// The --explore tag by itself gives an overview of how many trees there
    /// are at different cutoffss. When combined with a min_size, --explore
    /// outputs the number of tips in each tree.
    Split {
        #[structopt(
        short,
        long,
        help = "Don't split tree but print the number of trees at different cut-offs"
        )]
        explore: bool,
        #[structopt(short, long, help = "relax the minimum clade size so that the root subtree is a separate subtree.")]
        relaxed: bool,
        #[structopt(
        short,
        long,
        help = "the minimum clade size",
        required_if("explore", "true")
        )]
        min_size: Option<usize>,
    },
    /// Resolve polytomies with branches of 0 or nodes spread out between constraints
    Resolve {
        #[structopt(subcommand)]
        cmd: resolve::SubCommands,
    },
    ///Identify transmission lineages on a fully annotated tree
    TransmissionLineages{
        #[structopt(
        short,
        long,
        help = "name of the discrete annotation"
        )]
        key: String,
        #[structopt(
        short,
        long,
        help = "the deme for which introductions are being labeled"
        )]
        to: String,
        #[structopt(short, long, help = "output one row for each taxa")]
        taxa: bool,
    }
}

#[derive(Debug, StructOpt)]
pub struct Common {
    #[structopt(short, long,global =true, help = "tree is in nexus format")]
    nexus: bool,
    #[structopt(short, long, parse(from_os_str), help = "input tree file", global = true)]
    infile: Option<path::PathBuf>,
    // #[structopt(short, long, parse(from_os_str), help = "output tree file", global = true)]
    // outfile: Option<path::PathBuf>,
    // //TODO implement this log file option
    // #[structopt(short, long, parse(from_os_str), help = "logfile", global = true)]
    // logfile: Option<path::PathBuf>,
}

fn main() {
    env_logger::init();
    trace!("starting up");
    let args = Cli::from_args();
    debug!("{:?}", args);
    let start = std::time::Instant::now();
    let stdin = io::stdin();
    let result = match args.common.infile {

        Some(path) => {
            if args.common.nexus{
                let importer: NexusImporter<File> = nexus_importer::NexusImporter::from_reader(File::open(path).expect("issue with path "));
                run_commands(importer,args.cmd)
            }else{
                let importer = newick_importer::NewickImporter::from_reader(File::open(path).expect("issue with path "));
                run_commands(importer,args.cmd)
            }
        },
        None => {
         if args.common.nexus {
                let importer:NexusImporter<StdinLock> = nexus_importer::NexusImporter::from_reader(stdin.lock());
                run_commands(importer,args.cmd)

            }else{
               let importer = newick_importer::NewickImporter::from_reader(stdin.lock());
                run_commands(importer,args.cmd)

            }
        },
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

fn run_commands<R:std::io::Read,T:TreeImporter<R>>(tree_importer: T, cmd:Fertree) ->Result<(),Box<dyn Error>> {
    match cmd {
        Fertree::Stats { cmd } => stats::run(tree_importer, cmd),
        Fertree::Annotate { traits } => annotate::run(tree_importer, traits),
        Fertree::Extract { cmd } => extract::run(tree_importer, cmd),
        Fertree::Clades {cmd}=> clades::run(tree_importer, cmd),
        Fertree::Split {
            min_size,
            explore,
            relaxed,
        } => split::run(tree_importer, min_size, explore, !relaxed),
        Fertree::Resolve { cmd } => resolve::run(tree_importer, cmd),
        Fertree::TransmissionLineages{key,to,taxa}=>transmission_lineage::run(tree_importer,key,to,taxa),
    }
}