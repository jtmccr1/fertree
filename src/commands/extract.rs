use std::error::Error;
use structopt::StructOpt;

use rebl::io::parser::tree_importer::TreeImporter;
use rebl::tree::AnnotationValue;
use std::io::Write;

#[derive(Debug, StructOpt)]
pub enum SubCommands {
    /// Extract a list of the taxa names
    Taxa,
    /// Extract a tsv of the tip anotations
    Annotations,
    /// Extract a tree from a nexus file
    Tree {
        #[structopt(
            long,
            required_if("index", "None"),
            help = "the id of the tree to extract"
        )]
        id: Option<String>,
        #[structopt(
            long,
            required_if("id", "None"),
            help = "The 0 based index of the tree to extract."
        )]
        index: Option<usize>,
    },
}

pub fn run<R: std::io::Read, T: TreeImporter<R>>(
    trees: T,
    cmd: SubCommands,
) -> Result<(), Box<dyn Error>> {
    match cmd {
        SubCommands::Taxa => taxa(trees),
        SubCommands::Annotations => annotations(trees),
        SubCommands::Tree { id, index } => tree(trees, id, index),
    }
}

fn taxa<R: std::io::Read, T: TreeImporter<R>>(mut trees: T) -> Result<(), Box<dyn Error>> {
    let stdout = std::io::stdout(); // get the global stdout entity
    let mut handle = stdout.lock(); // acquire a lock on it
    while trees.has_tree() {
        let tree = trees.read_next_tree()?;
        let mut i = 0;
        while i < tree.get_external_node_count() {

            let tip = tree.get_external_node(i); 
            if let Some(taxa) = tree.get_taxon(tip){
                    writeln!(handle, "{}", taxa)?;
            
            }
            i += 1;
        }
    }
    Ok(())
}

fn annotations<R: std::io::Read, T: TreeImporter<R>>(mut trees: T) -> Result<(), Box<dyn Error>> {
    let stdout = std::io::stdout(); // get the global stdout entity
    let mut handle = stdout.lock(); // acquire a lock on it
    while trees.has_tree() {
        let tree = trees.read_next_tree()?;
        let header = tree
            .annotation_type
            .keys()
            .cloned()
            .collect::<Vec<String>>()
            .join("\t");
        writeln!(handle, "taxa\t{}", header)?;
        for node_ref in tree.external_nodes.iter() {
            let annotation_string = tree
                .annotation_type
                .keys()
                .map(|k| annotation_value_string(tree.get_annotation(*node_ref, k)))
                .collect::<Vec<String>>()
                .join("\t");
            if let Some(taxa) = tree.get_taxon(*node_ref) {
                writeln!(handle, "{}\t{}", taxa, annotation_string)?;
            } else {
                writeln!(handle, "\t{}", annotation_string)?;
            }
        }
    }
    Ok(())
}

fn annotation_value_string(value: Option<&AnnotationValue>) -> String {
    if let Some(annotation) = value {
        annotation.to_string()
    } else {
        "".to_string()
    }
}

fn tree<R: std::io::Read, T: TreeImporter<R>>(
    mut trees: T,
    id: Option<String>,
    index: Option<usize>,
) -> Result<(), Box<dyn Error>> {
    let stdout = std::io::stdout(); // get the global stdout entity
    let mut handle = stdout.lock(); // acquire a lock on it
    let mut found = false;

    if let Some(i) = index {
        let mut k = 0;
        while trees.has_tree() & !found {
            // let tree = trees.read_next_tree()?;
            if k == i {
                let tree = trees.read_next_tree()?;
                writeln!(handle, "{}", tree)?;
                found = true;
            }
            trees.skip_tree();
            k += 1;
        }
    } else if let Some(tree_id) = id {
        while trees.has_tree() & !found {
            let tree = trees.read_next_tree()?;
            if Some(tree_id.as_str()) == tree.get_id() {
                writeln!(handle, "{}", tree)?;
                found = true;
            }
        }
    };
    if !found {
        warn!("Tree not found");
    }
    Ok(())
}