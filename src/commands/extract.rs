use std::error::Error;
use rebl::tree::mutable_tree::MutableTree;
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
    ///Extract annotation transitions in tree
    Transitions{
        #[structopt(short, long, help = "name of the discrete annotation")]
        key: String,
    }
}

pub fn run<R: std::io::Read, T: TreeImporter<R>>(
    trees: T,
    cmd: SubCommands,
) -> Result<(), Box<dyn Error>> {
    match cmd {
        SubCommands::Taxa => taxa(trees),
        SubCommands::Annotations => annotations(trees),
        SubCommands::Tree { id, index } => tree(trees, id, index),
        SubCommands::Transitions{key}=>transitions(trees,key)
    }
}

fn taxa<R: std::io::Read, T: TreeImporter<R>>(mut trees: T) -> Result<(), Box<dyn Error>> {
    let stdout = std::io::stdout(); // get the global stdout entity
    let mut handle = stdout.lock(); // acquire a lock on it
    while trees.has_tree() {
        let tree = trees.read_next_tree()?;
        let mut i = 0;
        while i < tree.get_external_node_count() {
            if let Some(tip) = tree.get_external_node(i) {
                if let Some(taxa) = &tip.taxon {
                    writeln!(handle, "{}", taxa)?;
                }
            }
            i += 1;
        }
    }
    Ok(())
}

fn annotations<R: std::io::Read, T: TreeImporter<R>>(mut trees: T) -> Result<(), Box<dyn Error>> {
    let stdout = std::io::stdout(); // get the global stdout entity
    let mut handle = stdout.lock(); // acquire a lock on it
    let mut i =0;
    
    //get annotation keys from first tree
    let tree = trees.read_next_tree()?;
    let annotations = tree
        .annotation_type
        .keys()
        .cloned()
        .collect::<Vec<String>>();
    let header = annotations.join("\t");

     writeln!(handle, "tree\ttaxa\t{}", header)?;
        // process first tree
      for node_ref in tree.external_nodes.iter() {
            let annotation_string = annotations.iter()
                .map(|k| annotation_value_string(tree.get_annotation(*node_ref, k)))
                .collect::<Vec<String>>()
                .join("\t");
            if let Some(taxa) = tree.get_taxon(*node_ref) {
                writeln!(handle, "{}\t{}\t{}", i, taxa, annotation_string)?;
            } else {
                writeln!(handle, "{}\t\t{}", i,annotation_string)?;
            }
        }
        i+=1;


    while trees.has_tree() {
        let tree = trees.read_next_tree()?;
        for node_ref in tree.external_nodes.iter() {
            let annotation_string = annotations.iter()
                .map(|k| annotation_value_string(tree.get_annotation(*node_ref, k)))
                .collect::<Vec<String>>()
                .join("\t");
            if let Some(taxa) = tree.get_taxon(*node_ref) {
                writeln!(handle, "{}\t{}\t{}", i, taxa, annotation_string)?;
            } else {
                writeln!(handle, "{}\t\t{}", i,annotation_string)?;
            }
        }
        i+=1;
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

struct Transition {
    id: usize,
    source: String,
    destination:String,
    time:f64

}

fn transitions<R: std::io::Read, T: TreeImporter<R>>(mut trees: T,    key: String) -> Result<(), Box<dyn Error>> {
    let stdout = std::io::stdout(); // get the global stdout entity
    let mut handle = stdout.lock(); // acquire a lock on it
     writeln!(
            handle,
            "tree\tsource\tdestination\theight"
        )?;
        let mut i = 0;
    while trees.has_tree() {
         let mut tree = trees.read_next_tree()?;

        // if let Some(most_recent_sample) = origin {
        //     tree.calc_relative_node_heights(most_recent_sample);
        // } else {
            tree.calc_node_heights();
        // }

        

        let transitions:Vec<Transition> = get_transitions(&tree,&key);
        for transition in transitions {
            writeln!(
                handle,
                "{}\t{}\t{}\t{}",
                i,
                transition.source,
                transition.destination,
                transition.time
            )?;
        }
        i += 1;

    }
    Ok(())
}

fn get_transitions(tree: &MutableTree,key: &String)->Vec<Transition> {
    let mut transitions:Vec<Transition> = vec![];

    traverse(tree,tree.get_root().unwrap(),key,&mut transitions);

    return transitions

}
fn traverse(tree: &MutableTree, node: usize,key: &String,transitions:&mut Vec<Transition>){
    if let Some(value) = tree.get_annotation(node, key){
        for child in tree.get_children(node){
           let child_annotation = tree.get_annotation(child, key).unwrap_or_else(||  panic!("All nodes must be annotated. found a node without {}", key));
           if child_annotation!=value {
            transitions.push( Transition { id: transitions.len(), source: value.to_string(), destination: child_annotation.to_string(), time: tree.get_height(child).unwrap() })
           }
           traverse(tree, child, key,transitions)
        }
    }else{
        panic!("All nodes must be annotated. found a node without {}", key)
    }
}