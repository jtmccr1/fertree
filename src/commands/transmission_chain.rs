

// use super:: command_io;
// use rebl::tree::mutable_tree::MutableTree;
use rebl::{io::parser::tree_importer::TreeImporter, tree::mutable_tree::MutableTree};
use std::error::Error;
use std::io::Write;


// This command will identity the transmission network underlying a fully sampled remaster seir tree.
// For each node will assign an individual.
// we will start at the tips and traverse backwards until we hit the root or a node with annotation type=E node (this will also get the individual's label)
pub fn run<R:std::io::Read, T:TreeImporter<R>>(
    mut trees:T
) -> Result<(),Box<dyn Error>>{
    let stdout = std::io::stdout(); // get the global stdout entity
    let mut handle = stdout.lock(); // acquire a lock on it

    while trees.has_tree() {
        let mut tree = trees.read_next_tree()?;
        //TODO avoid parsing at each loop
        annotate_nodes(&mut tree)?;
        writeln!(handle, "{}", tree)?;
    }
    Ok(())
}
// This command will identity the transmission network underlying a fully sampled remaster seir tree.
// For each node will assign an individual.
// we will start at the tips and traverse backwards until we hit the root or a node with annotation type=E node (this will also get the individual's label)

fn annotate_nodes(tree: &mut MutableTree) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Implement the logic to annotate nodes
    for i in 0..tree.get_external_node_count(){
        // Your logic here for each external node
        
        let tip =tree.get_external_node(i).unwrap();

        // let tip_number = tip.number;
        let mut node = tip.number;
        let taxon = tip.taxon.as_ref().unwrap().clone();

        tree.annotate_node(node, "id".to_string(), rebl::tree::AnnotationValue::Discrete(taxon.to_string()));
        
        
        while let Some(parent) = tree.get_parent(node){
            if let Some(node_type) = tree.get_annotation(parent, "type") {
                let node_type_str = node_type.to_string();
                let re = regex::Regex::new(r"^E").unwrap();
                if re.is_match(&node_type_str) || tree.get_root()==Some(parent){
                    tree.annotate_node(parent, "id".to_string(), rebl::tree::AnnotationValue::Discrete(taxon.to_string()));
                    break;
                } else {
                    tree.annotate_node(parent, "id".to_string(), rebl::tree::AnnotationValue::Discrete(taxon.to_string()));
                    node = parent;
                }
            
            }else{
                    panic!("Found a node that did not have a value for {} annotation","type")
                }
            }
    
}
    Ok(())
}