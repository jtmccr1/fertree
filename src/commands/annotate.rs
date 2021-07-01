use super::command_io;

use rebl::tree::mutable_tree::MutableTree;
use rebl::tree::AnnotationValue;
use std::collections::HashMap;
use std::error::Error;
use std::io::Write;
use std::path;

use csv::Reader;
use std::fs::File;
use rebl::io::parser::tree_importer::TreeImporter;

pub fn run<R: std::io::Read, T: TreeImporter<R>>(mut trees: T,
                                                 traits: path::PathBuf,
) -> Result<(), Box<dyn Error>> {
    let stdout = std::io::stdout(); // get the global stdout entity
    let mut handle = stdout.lock(); // acquire a lock on it

    while trees.has_tree() {
        let mut tree = trees.read_next_tree()?;
        //TODO avoid parsing at each loop
        let mut reader = command_io::parse_tsv(&traits)?;
        annotate_nodes(&mut tree, &mut reader)?;
        writeln!(handle, "{}", tree)?;
    }
    Ok(())
}

pub fn annotate_nodes(
    tree: &mut MutableTree,
    reader: &mut Reader<File>,
) -> Result<(), Box<dyn Error>> {
    //todo fix to handle taxa differently
    type Record = HashMap<String, Option<AnnotationValue>>;

    let header = reader.headers()?;
    let taxon_key = header.get(0).unwrap().to_string();

    for result in reader.deserialize() {
        trace!("{:?}",result);
        let record: Record = result?;
        if let Some(AnnotationValue::Discrete(taxon)) = record.get(&*taxon_key).unwrap() {
            if let Some(node_ref) = tree.get_label_node(&taxon) {
                for (key, value) in record {
                    if key != taxon_key {
                        if let Some(annotation_value) = value {
                            tree.annotate_node(node_ref, key, annotation_value)
                        }
                    }
                }
            } else {
                warn!("Node {} not found in tree", taxon)
            }
        }
    }
    Ok(())
}

