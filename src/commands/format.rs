use std::error::Error;
use structopt::StructOpt;

use rebl::io::parser::tree_importer::TreeImporter;
use rebl::tree::AnnotationValue;
use std::io::{Write, BufWriter};
use std::fs::File;

#[derive(Debug, StructOpt)]
pub enum SubCommands {
    /// Nexus format
    nexus,
    /// Newick
    newick,
}

pub fn run<R: std::io::Read, T: TreeImporter<R>>(
    trees: T,
    cmd: SubCommands,
) -> Result<(), Box<dyn Error>> {
    match cmd {
        SubCommands::nexus => {
            println!("nexus not implemented");
            Ok(())
        },
        SubCommands::newick =>{
            println!("newick not implemented");
            Ok(())
        } ,
    }
}

