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
        s.push_str(tree.get_node(node_ref).unwrap().taxon.as_ref().unwrap().as_str());
    } else {
        s.push_str("(");
        tree.get_children(node_ref).iter()
            .map(|child| write_node(tree, *child))
            .collect::<Vec<String>>().join(",");
        s.push_str(")");
    }
    s.push_str(write_annotations(tree, node_ref).as_str());
    if let Some(label) = tree.get_node_label(node_ref) {
        s.push_str(label)
    }
    s.push_str(":");
    s.push_str(tree.get_length(node_ref).unwrap().to_string().as_str());
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
