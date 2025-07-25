mod commands;

// use commands::{split, transmission_lineage};
// use commands::{annotate, clades, extract, resolve, stats};
use rebl::io::parser::nexus_importer::NexusImporter;
use rebl::io::parser::tree_importer::TreeImporter;
use rebl::io::parser::{newick_importer, nexus_importer};
use std::error::Error;
use std::fs::File;
use std::io::StdinLock;
use std::{io, path};
use structopt::StructOpt;

#[macro_use]
extern crate log;
extern crate rebl;

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
    ///Change the format of a tree
    Format{
        #[structopt(subcommand)]
        cmd:commands::format::SubCommands,

    },
    /// A few useful stats about the trees
    Stats {
        #[structopt(subcommand)]
        cmd: Option<commands::stats::SubCommands>,
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
        cmd: commands::extract::SubCommands,
    },
    /// Collapse (i.e. subsample) monophyletic clades into a set number of tips
    Clades {
        #[structopt(subcommand)]
        cmd: commands::clades::SubCommands,
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
        #[structopt(
            short,
            long,
            help = "relax the minimum clade size so that the root subtree is a separate subtree."
        )]
        relaxed: bool,
        #[structopt(
            short,
            long,
            help = "the minimum clade size",
            required_if("explore", "true")
        )]
        min_size: Option<usize>,
    },
    /// Sample n tips from a tree .
    ///
    Prune {
        #[structopt(subcommand)]
        cmd: commands::prune::SubCommands,
    },
    /// Resolve polytomies with branches of 0 or nodes spread out between constraints
    Resolve {
        #[structopt(subcommand)]
        cmd: commands::resolve::SubCommands,
    },
    ///Identify transmission lineages on a fully annotated tree. Unannotated nodes will be assigned
    ///an "unknown" as a discrete trait.
    TransmissionLineages {
        #[structopt(
            long,
            parse(from_os_str),
            help = "file of taxa to ignore",
            global = true
        )]
        ignore_taxa: Option<path::PathBuf>,
        #[structopt(short, long, help = "name of the discrete annotation")]
        key: String,
        #[structopt(
            short,
            long,
            help = "the deme for which introductions are being labeled"
        )]
        to: String,
        #[structopt(
            long,
            help = "Include a semicolon separated list of the taxa in each introduction"
        )]
        taxa: bool,
        #[structopt(
            short,
            long,
            help = "most recent time of sampling used to calculate node heights in time. defaults to 0 with heights increasing towards the root. "
        )]
        origin: Option<f64>,
        #[structopt(
            short,
            long,
            help = "the earliest time allowed for an introduction. \
        Any inferred introduction before this time will be passed down to children until an node with an acceptable time is found."
        )]
        cutoff: Option<f64>,
        #[structopt(
            short,
            long,
            help = "the maximum detection lag for an introduction. this is the time \
        between any ancestor in the deme and the next sample. Any introduction with lag grater than \
        this limit is split into introductions that respect the lag are found."
        )]
        lag: Option<f64>,
    },
    /// Commands to modify branch lengths
    Brlen {
        #[structopt(subcommand)]
        cmd: commands::branchlengths::SubCommands,
    },
    /// annotate nodes with taxon ids assuming an sier model from reMaster.
    /// Nodes are labeled from the tips until an E type is hit (inclusive)
    TransmissionChain{

    }
}

#[derive(Debug, StructOpt)]
pub struct Common {
    #[structopt(short, long, global = true, help = "tree is in nexus format")]
    nexus: bool,
    #[structopt(
        short,
        long,
        parse(from_os_str),
        help = "input tree file",
        global = true
    )]
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
            if args.common.nexus {
                let importer: NexusImporter<File> = nexus_importer::NexusImporter::from_reader(
                    File::open(path).expect("issue with path "),
                );
                run_commands(importer, args.cmd)
            } else {
                let importer = newick_importer::NewickImporter::from_reader(
                    File::open(path).expect("issue with path "),
                );
                run_commands(importer, args.cmd)
            }
        }
        None => {
            if args.common.nexus {
                let importer: NexusImporter<StdinLock> =
                    nexus_importer::NexusImporter::from_reader(stdin.lock());
                run_commands(importer, args.cmd)
            } else {
                let importer = newick_importer::NewickImporter::from_reader(stdin.lock());
                run_commands(importer, args.cmd)
            }
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

fn run_commands<R: std::io::Read, T: TreeImporter<R>>(
    tree_importer: T,
    cmd: Fertree,
) -> Result<(), Box<dyn Error>> {
    match cmd {
        Fertree::Format { cmd } => commands::format::run(tree_importer, cmd),
        Fertree::Stats { cmd } => commands::stats::run(tree_importer, cmd),
        Fertree::Annotate { traits } => commands::annotate::run(tree_importer, traits),
        Fertree::Extract { cmd } => commands::extract::run(tree_importer, cmd),
        Fertree::Clades { cmd } => commands::clades::run(tree_importer, cmd),
        Fertree::Split {
            min_size,
            explore,
            relaxed,
        } => commands::split::run(tree_importer, min_size, explore, !relaxed),
        Fertree::Resolve { cmd } => commands::resolve::run(tree_importer, cmd),
        Fertree::Brlen { cmd } => commands::branchlengths::run(tree_importer, cmd),
        Fertree::Prune { cmd } => commands::prune::run(tree_importer, cmd),
        Fertree::TransmissionLineages {
            key,
            ignore_taxa,
            to,
            taxa,
            origin,
            cutoff,
            lag,
        } => commands::transmission_lineage::run(
            tree_importer,
            ignore_taxa,
            key,
            to,
            taxa,
            origin,
            cutoff,
            lag,
        ),
        Fertree::TransmissionChain {  }=> commands::transmission_chain::run(tree_importer)
    }
}
