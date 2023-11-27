use rebl::io::parser::tree_importer::TreeImporter;
use std::error::Error;
use std::io::Write;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum SubCommands {
    //Get a table of the patristic distances between all pairs of tips in the tree
    // Patristic,
    //Get the root to tip distance for each tip in the tree
    Divergence,
    //for a given tip, how much of its divergence is shared with each other tip.
    CovEvol 
}

fn divergence<R: std::io::Read, T: TreeImporter<R>>(mut trees: T) -> Result<(), Box<dyn Error>> {
    let stdout = std::io::stdout(); // get the global stdout entity
    let mut handle = stdout.lock(); // acquire a lock on it
    writeln!(handle, "tree\ttaxa\tdivergence")?;
    let mut t = 0; //TODO use id if in tree maybe every tree gets an id in parser
    while trees.has_tree() {
        let mut tree = trees.read_next_tree()?;
        tree.calc_node_heights();
        let root_height = tree.get_height(tree.get_root().unwrap()).expect("Heights should be calculated");
        for i in 0..tree.get_external_node_count() {
            let tip = tree.get_external_node(i);
            let taxa = tree.get_taxon(tip).unwrap_or("");
            let height = tree.get_height(i).expect("Heights should be calculated");
            let div = root_height - height;
            writeln!(handle, "{}\t{}\t{}", t, taxa, div)?;
        }
        t += 1;
    }

    Ok(())
}

fn covEvol<R: std::io::Read, T: TreeImporter<R>>(mut trees: T) -> Result<(), Box<dyn Error>> {
    let stdout = std::io::stdout(); // get the global stdout entity
    let mut handle = stdout.lock(); // acquire a lock on it
    writeln!(handle, "tree\ttaxa1\ttaxa2\tshared")?;
    let mut t = 0; //TODO use id if in tree maybe every tree gets an id in parser
    while trees.has_tree() {
        let mut tree = trees.read_next_tree()?;
        tree.calc_node_heights();
        let root_height = tree.get_height(tree.get_root().expect("tree must be rooted")).expect("Heights should be calculated");
        for i in 0..tree.get_external_node_count() {
            for j in 0..tree.get_external_node_count() {
                let tip1 = tree.get_external_node(i);
                let tip2 = tree.get_external_node(j);
                if(tip1 == tip2){
                    continue;
                }
                let taxa1 = tree.get_taxon(tip1).unwrap_or("");
                let taxa2 = tree.get_taxon(tip2).unwrap_or("");
                let height = tree.get_height(tip1).expect("Heights should be calculated");
                let mrca = tree.get_mrca(tip1,tip2);
                let mrca_height = tree.get_height(mrca).expect("Heights should be calculated");
                let proportion_independent =  (root_height - mrca_height)/(root_height - height) ;

                writeln!(handle, "{}\t{}\t{}\t{}", t, taxa1, taxa2, proportion_independent)?;
            }
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
        Some(SubCommands::Divergence) => divergence(trees),
        Some(SubCommands::CovEvol) => covEvol(trees),
        None => {
            eprintln!("No command given");
            Ok(())
        }
    }
}
