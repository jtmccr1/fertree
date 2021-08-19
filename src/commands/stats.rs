use rebl::io::parser::tree_importer::TreeImporter;
use std::error::Error;
use std::io::Write;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum SubCommands {
    Nodes,
}

fn general_stats<R: std::io::Read, T: TreeImporter<R>>(mut trees: T) -> Result<(), Box<dyn Error>> {
    let stdout = std::io::stdout(); // get the global stdout entity
    let mut handle = stdout.lock(); // acquire a lock on it
    writeln!(handle, "nodes\ttips\trootHeight\tsumbl\tmeanbl")?;

    while trees.has_tree() {
        let mut tree = trees.read_next_tree()?;
        let root = tree.get_root().unwrap();
        let nodes = tree.get_node_count();
        // let internal = tree.get_internal_node_count();
        let tips = tree.get_external_node_count();
        let mut bl = Vec::with_capacity(tree.get_node_count());
        bl.resize(tree.get_node_count(), 0.0);
        for node_ref in tree.preorder_iter() {
            if node_ref != tree.get_root().expect("stats assume rooted nodes") {
                if let Some(node) = tree.get_node(node_ref) {
                    if let Some(length) = node.length {
                        bl[node_ref] = length;
                    }
                }
            }
        }
        let sum_bl = bl.iter().fold(0.0, |acc, x| acc + x);
        let mean_bl = sum_bl / ((tree.get_node_count() as f64) - 1.0); //no branch on root
        tree.calc_node_heights();
        let root_height = tree.get_height(root).unwrap();
        writeln!(
            handle,
            "{}\t{}\t{:.2e}\t{:.2e}\t{:.2e}",
            nodes, tips, root_height, sum_bl, mean_bl
        )?;
    }
    Ok(())
}

fn nodes<R: std::io::Read, T: TreeImporter<R>>(mut trees: T) -> Result<(), Box<dyn Error>> {
    let stdout = std::io::stdout(); // get the global stdout entity
    let mut handle = stdout.lock(); // acquire a lock on it
    writeln!(handle, "tree\theight\tlength\ttaxa")?;
    let mut t = 0; //TODO use id if in tree maybe every tree gets an id in parser
    while trees.has_tree() {
        let mut tree = trees.read_next_tree()?;
        tree.calc_node_heights();
        for i in 0..tree.get_node_count() {
            let taxa = tree.get_taxon(i).unwrap_or("");
            let height = tree.get_height(i).expect("Heights should be calculated");
            let mut length = f64::NAN;
            if let Some(p) = tree.get_parent(i) {
                length = tree.get_height(p).expect("Heights should be calculated") - height;
            }
            writeln!(handle, "{}\t{}\t{}\t{}", t, height, length, taxa)?;
        }
        t += 1;
    }

    Ok(())
}

pub fn run<R: std::io::Read, T: TreeImporter<R>>(
    trees: T,
    cmd: Option<SubCommands>,
) -> Result<(), Box<dyn Error>> {
    //TODO move tree reading and output buffer handling out here and pass to commands

    match cmd {
        None => general_stats(trees),
        Some(SubCommands::Nodes) => nodes(trees),
    }
}
