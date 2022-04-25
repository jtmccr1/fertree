use rand::prelude::SliceRandom;
use rand::{seq::IteratorRandom, thread_rng};
use rebl::io::parser::tree_importer::TreeImporter;
use rebl::tree::mutable_tree::MutableTree;
use std::collections::HashSet;
use std::error::Error; // 0.6.1
use std::fs::File;
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::path;
use structopt::StructOpt;
// use rayon::prelude::*;

#[derive(Debug, StructOpt)]
pub enum SubCommands {
    /// randomly sample n tips from a tree
    Sample {
        #[structopt(short, long, help = "sample same tips from all trees")]
        all: bool,
        #[structopt(short, long, help = "number of tips to keep")]
        n: usize,
        #[structopt(short,long, help = "include all ancestral nodes for original tree")]
        keep_single_children: bool, 
    },
    /// prune tree to just provided tips
    Keep {
        #[structopt(short, long, parse(from_os_str), help = "text file with taxa to keep")]
        taxon_list: path::PathBuf,
        #[structopt(short,long, help = "include all ancestral nodes for original tree")]
        keep_single_children: bool,
    },

    /// prune provided tips from tree
    Remove {
        #[structopt(short, long, parse(from_os_str), help = "text file with taxa to keep")]
        taxon_list: path::PathBuf,
        #[structopt(short,long, help = "include all ancestral nodes for original tree")]
        keep_single_children: bool,
    },
}

pub fn run<R: std::io::Read, T: TreeImporter<R>>(
    mut trees: T,
    cmd: SubCommands,
) -> Result<(), Box<dyn Error>> {
    let stdout = std::io::stdout(); // get the global stdout entity
    let mut handle = stdout.lock(); // acquire a lock on it
    let mut rng = thread_rng();
    let mut taxa: HashSet<String> = HashSet::new();

    // let mut tree = trees.read_next_tree()?;
    match cmd {
        SubCommands::Sample { n, ref all , keep_single_children:keepSingleChildren} => {
            while trees.has_tree() {
                let mut tree = trees.read_next_tree()?;
                if !all || taxa.is_empty() {
                    taxa = tree
                        .external_nodes
                        .choose_multiple(&mut rng, n)
                        // .iter()
                        .map(|nref| tree.get_taxon(*nref))
                        .map(|n| String::from(n.unwrap()))
                        .collect();
                }
                debug!("{:?}", taxa);
                let new_tree = if keepSingleChildren==true {MutableTree::get_ancestral_tree(&mut tree, &taxa)}else{MutableTree::from_tree(&mut tree, &taxa)};
                writeln!(handle, "{}", new_tree)?;
            }
        }
        SubCommands::Keep {
            taxon_list: taxonList,
            keep_single_children:keepSingleChildren
        } => {
            let file = BufReader::new(File::open(&taxonList)?);
            taxa = file.lines().map(|x| x.unwrap()).collect();
            while trees.has_tree() {
                let mut tree = trees.read_next_tree()?;
                let new_tree = if keepSingleChildren ==true {MutableTree::get_ancestral_tree(&mut tree, &taxa)} else {MutableTree::from_tree(&mut tree, &taxa)};
                writeln!(handle, "{}", new_tree)?;
            }
        }
        SubCommands::Remove {
            taxon_list: taxonList,
            keep_single_children:keepSingleChildren
        } => {
            let file = BufReader::new(File::open(&taxonList)?);
            taxa = file.lines().map(|x| x.unwrap()).collect();
            while trees.has_tree() {
                let mut tree = trees.read_next_tree()?;
                let mut taxa_to_keep: HashSet<String> = tree
                    .external_nodes
                    .iter()
                    .map(|nref| tree.get_taxon(*nref))
                    .map(|n| String::from(n.unwrap()))
                    .collect::<HashSet<String>>();

                taxa_to_keep.retain(|s| taxa.contains(s));
                let new_tree = if keepSingleChildren {MutableTree::get_ancestral_tree(&mut tree, &taxa_to_keep)} else {MutableTree::from_tree(&mut tree, &taxa_to_keep)};
                writeln!(handle, "{}", new_tree)?;
            }
        }
    }

    Ok(())
}
