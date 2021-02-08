use crate::tree::mutable_tree::{MutableTree, TreeIndex};
use crate::tree::AnnotationValue;
use std::fmt;

impl fmt::Display for MutableTree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let start = std::time::Instant::now();
        let s = write_newick(self);
        trace!("Tree string built in {} seconds ",start.elapsed().as_secs());
        write!(f,"{}",s)
    }
}


fn write_newick(tree: &MutableTree) -> String {
    //TODO check if branchlengths known and throw error if not known
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
            .filter(|k|tree.get_annotation(node_ref,k).is_some())
            .map(|k| write_annotation(k,tree.get_annotation(node_ref,k)))
            .collect::<Vec<String>>()
            .join(",");
        if annotation_string.len()>0 {
            s.push_str("[&");
            s.push_str(annotation_string.as_str());
            s.push_str("]");
        }
    }
    s
}

pub fn write_annotation(key: &String, value: Option<&AnnotationValue>) -> String {
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
    use crate::io::parser::newick_parser::NewickParser;

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
        assert_eq!("((:1,:2):0.1,:1);",tree.to_string());
    }
    #[test]
    fn tree_with_annotations(){
        let s = "((A[&location=UK]:0.3,B[&location=USA]:0.05):0.9,C[&location=US]:0.1);";
        let root = NewickParser::parse_tree(s).expect("error in parsing");
        let tree = MutableTree::from_fixed_node(root);
        assert_eq!(s,tree.to_string())
    }
        #[test]
    fn tree_with_label(){
        let s = "((A[&location=UK]:0.3,B[&location=USA]:0.05)label:0.9,C[&location=US]:0.1);";
        let root = NewickParser::parse_tree(s).expect("error in parsing");
        let tree = MutableTree::from_fixed_node(root);
            println!("{:?}", tree.get_internal_node(1));
        assert_eq!(s,tree.to_string())
    }

    #[test]
    fn tree_with_quotes(){
        let s = "((A[&location=UK]:0.3,B[&location=USA]:0.05):0.9,'C d'[&location=US]:0.1);";
        let root = NewickParser::parse_tree(s).expect("error in parsing");
        let tree = MutableTree::from_fixed_node(root);
        assert_eq!(s,tree.to_string())
    }

}
