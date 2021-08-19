use crate::tree::mutable_tree::{MutableTree, TreeIndex};
use crate::tree::AnnotationValue;
use std::fmt;

impl fmt::Display for MutableTree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = write_newick(self);
        write!(f, "{}", s)
    }
}

fn write_newick(tree: &MutableTree) -> String {
    //TODO check if branchlengths known and throw error if not known. Need io erro
    if !tree.branchlengths_known {
        panic!("tried to write  a tree without branchlengths known! calculate them first!")
    }
    let mut s = write_node(tree, tree.get_root().unwrap());
    s.push(';');
    s
}

//TODO write without annotation

fn write_node(tree: &MutableTree, node_ref: TreeIndex) -> String {
    let mut s = String::new();
    if tree.is_external(node_ref) {
        if let Some(taxon_string) = tree.get_taxon(node_ref) {
            let quoted = taxon_string.contains(char::is_whitespace);
            if quoted {
                s.push('\'')
            }
            s.push_str(taxon_string);
            if quoted {
                s.push('\'')
            }
        }
    } else {
        s.push('(');
        let children_string = tree
            .get_children(node_ref)
            .iter()
            .map(|child| write_node(tree, *child))
            .collect::<Vec<String>>()
            .join(",");
        s.push_str(&children_string);
        s.push(')');
    }
    s.push_str(write_annotations(tree, node_ref).as_str());
    if let Some(label) = tree.get_node_label(node_ref) {
        s.push_str(label);
    }
    if let Some(l) = tree.get_length(node_ref) {
        s.push(':');
        let length = if l < 1e-4 {
            format!("{:e}", l)
        } else {
            l.to_string()
        };
        s.push_str(length.as_str());
    }
    s
}

fn write_annotations(tree: &MutableTree, node_ref: TreeIndex) -> String {
    let mut s = String::new();
    let keys = tree.get_annotation_keys();
    if keys.len() > 0 {
        let annotation_string = keys
            .filter(|k| tree.get_annotation(node_ref, k).is_some())
            .map(|k| write_annotation(k, tree.get_annotation(node_ref, k)))
            .collect::<Vec<String>>()
            .join(",");
        if !annotation_string.is_empty() {
            s.push_str("[&");
            s.push_str(annotation_string.as_str());
            s.push(']');
        }
    }
    s
}

pub fn write_annotation(key: &str, value: Option<&AnnotationValue>) -> String {
    if let Some(annotation) = value {
        let value_string = match annotation {
            AnnotationValue::Discrete(_) => {
                "\"".to_string() + annotation.to_string().as_str() + "\""
            }
            _ => annotation.to_string(),
        };
        format!("{}={}", key, value_string)
    } else {
        "".to_string()
    }
}

#[cfg(test)]
mod tests {
    use crate::io::parser::newick_importer::NewickImporter;
    use crate::tree::fixed_tree::FixedNode;
    use crate::tree::mutable_tree::MutableTree;
    use std::io::BufReader;

    #[test]
    fn basic_tree() {
        let mut tip1 = FixedNode::new();
        tip1.length = Some(1.0);
        let mut tip2 = FixedNode::new();
        tip2.length = Some(2.0);
        let mut tip3 = FixedNode::new();
        tip3.length = Some(1.0);

        let mut internal1 = FixedNode::new();
        internal1.length = Some(0.1);
        internal1.children = vec![tip1, tip2];
        let mut root = FixedNode::new();
        root.children = vec![internal1, tip3];

        let tree = MutableTree::from_fixed_node(root);
        assert_eq!("((:1,:2):0.1,:1);", tree.to_string());
    }
    #[test]
    fn tree_with_quoted_annotations() {
        let s =
            "((A[&location=\"UK\"]:0.3,B[&location=\"USA\"]:0.05):0.9,C[&location=\"US\"]:0.1);";
        let tree =
            NewickImporter::read_tree(BufReader::new(s.as_bytes())).expect("error in parsing");
        assert_eq!(tree.to_string(), s)
    }
    #[test]
    fn tree_with_unquoted_annotations() {
        let s = "((A[&location=UK]:0.3,B[&location=USA]:0.05):0.9,C[&location=US]:0.1);";
        let tree =
            NewickImporter::read_tree(BufReader::new(s.as_bytes())).expect("error in parsing");
        let exp =
            "((A[&location=\"UK\"]:0.3,B[&location=\"USA\"]:0.05):0.9,C[&location=\"US\"]:0.1);";

        assert_eq!(tree.to_string(), exp)
    }
    #[test]
    fn tree_with_label() {
        let s = "((A:0.3,B:0.05)label:0.9,C:0.1);";
        let tree =
            NewickImporter::read_tree(BufReader::new(s.as_bytes())).expect("error in parsing");
        println!("{:?}", tree.get_internal_node(1));
        let exp = "((A:0.3,B:0.05)label:0.9,C:0.1);";

        assert_eq!(exp, tree.to_string())
    }

    #[test]
    fn tree_with_quotes() {
        let s = "((A[&location=UK]:0.3,B[&location=USA]:0.05):0.9,'C d'[&location=US]:0.1);";
        let tree =
            NewickImporter::read_tree(BufReader::new(s.as_bytes())).expect("error in parsing");
        let exp = "((A[&location=\"UK\"]:0.3,B[&location=\"USA\"]:0.05):0.9,'C d'[&location=\"US\"]:0.1);";

        assert_eq!(tree.to_string(), exp)
    }
}
