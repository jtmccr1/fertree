use crate::tree::mutable_tree::{MutableTree, TreeIndex};
use crate::io::parser::newick_parser::AnnotationValue;

fn write_newick(tree: &MutableTree) -> String {
    let mut s = write_node(tree,tree.get_root().unwrap());
    s.push_str(";");
    s
}

fn write_node(tree: &MutableTree, node_ref: TreeIndex) ->String{
    let mut s =  String::new();
    if tree.is_external(node_ref) {
        if let Some(taxon_string) = tree.get_taxon(node_ref){
            s.push_str(taxon_string);
        }

    } else {
        s.push_str("(");
        let children_string = tree.get_children(node_ref).iter()
            .map(|child| write_node(tree, *child))
            .collect::<Vec<String>>().join(",");
        s.push_str(&children_string);
        s.push_str(")");
    }
    s.push_str(write_annotations(tree, node_ref).as_str());
    if let Some(label) = tree.get_node_label(node_ref) {
        s.push_str(label)
    }
    if let Some(l) =tree.get_length(node_ref){
        s.push_str(":");
        s.push_str(l.to_string().as_str());
    }
    return s;
}

fn write_annotations(tree: &MutableTree, node_ref: TreeIndex) ->String{
    let mut s = String::new();
    let keys = tree.get_annotation_keys();
    if keys.len() > 0 {

        let annotation_string = keys
            .map(|k| write_annotation(k,tree.get_annotation(node_ref,k)))
            .collect::<Vec<String>>()
            .join(",");
        s.push_str("[&");
        s.push_str(annotation_string.as_str());
        s.push_str("]");
    }
    s
}

fn write_annotation(key: &String, value: Option<&AnnotationValue>) -> String {
    if let Some(annotation) = value {
        let value_string =annotation.to_string();
        format!("{}={}", key, value_string)
    }else {
        "".to_string()
    }
}

#[cfg(test)]
mod tests {
    use crate::tree::fixed_tree::FixedNode;
    use crate::tree::mutable_tree::MutableTree;
    use crate::io::writer::newick_writer::{write_newick, write_node};

    #[test]
    fn basic_tree(){
        let mut tip1 = FixedNode::new();
        tip1.length = Some(1.0);
        let mut tip2 = FixedNode::new();
        tip2.length = Some(2.0);
        let mut tip3 =FixedNode::new();
        tip3.length = Some(1.0);

        let mut internal1 = FixedNode::new();
        internal1.length = Some(0.1);
        internal1.children = vec![Box::new(tip1),Box::new(tip2)];
        let mut root = FixedNode::new();
        root.children = vec![Box::new(internal1), Box::new(tip3)];

        let tree = MutableTree::from_fixed_node(root);

        let internal = tree.get_internal_node(1).unwrap();
        println!("{:?}", tree.is_external(internal.number));
        println!("{:?}", internal);
        println!("{}", write_node(&tree,internal.number ));
        println!("{}",write_newick(&tree));

    }

}
