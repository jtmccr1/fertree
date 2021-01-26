use std::option::Option;
use super::fixed_tree::FixedNode;
use std::collections::HashMap;
use crate::parsers::newick_parser::AnnotationValue;

pub type TreeIndex = usize;


pub struct MutableTreeNodeReference {
    taxon: Option<String>,
    parent: Option<TreeIndex>,
}

#[derive(Debug)]
pub struct MutableTreeNode {
    pub taxon: Option<String>,
    pub label: Option<String>,
    pub parent: Option<TreeIndex>,
    pub first_child: Option<TreeIndex>,
    pub next_sibling: Option<TreeIndex>,
    pub previous_sibling: Option<TreeIndex>,
    pub length: Option<f64>,
    number: Option<usize>,

}

impl MutableTreeNode {
    pub(crate) fn new(taxon: Option<String>,
                      parent: Option<TreeIndex>,
                      number:Option<usize>) -> Self {
        MutableTreeNode { taxon: taxon, label: None, parent: parent, first_child: None, next_sibling: None, previous_sibling: None, length: None,  number }
    }
}

pub struct MutableTree {
    pub nodes: Vec<Option<MutableTreeNode>>,
    pub node_annotations:Vec<Option<HashMap<String,AnnotationValue>>>,
    pub external_nodes:Vec<Option<TreeIndex>>,
    pub internal_nodes:Vec<Option<TreeIndex>>,
    root: Option<TreeIndex>,
}

impl MutableTree {
    pub fn new(root: FixedNode) ->Self{
       let mut tree = MutableTree {
           nodes: Vec::new(),
           node_annotations: vec![],
           external_nodes:Vec::new(),
           internal_nodes:Vec::new(),
           root: None,
        };
        tree.new_helper(root, None);
        tree.set_root(Some(0));
        return tree
    }
    fn new_helper(&mut self, node: FixedNode, parent:Option<TreeIndex>){
        let index = self.nodes.len();
        self.add_node(MutableTreeNode::new(node.taxon,parent,Some(index)));
        self.node_annotations.push(node.annotations);
        if node.children.len()>0{
            self.internal_nodes.push(Some(index));
        }else{
            self.external_nodes.push(Some(index));
        }
        for child in node.children{
            self.new_helper(*child, Some(index));
        }
    }
    pub fn iter(&self) -> PreorderIter {
        PreorderIter::new(self.root)
    }
    pub fn set_root(&mut self, root: Option<TreeIndex>) {
        self.root = root
    }

    fn add_node(&mut self, node: MutableTreeNode) -> TreeIndex {
        let index = self.nodes.len();
        self.nodes.push(Some(node));
        let child = self.get_node(index).expect("no way we hit this");
        if let Some(parent) = child.parent {
            self.add_child(parent, index);
        }
        return index;
    }


    pub fn add_child(&mut self, parent: TreeIndex, child: TreeIndex) {
        let mut children = self.get_children(parent);
        if let Some(last_child) = children.pop() {
            let sibling = self.get_node_mut(last_child).expect("sibling not tree");
            sibling.next_sibling = Some(child);
            let child_node = self.get_node_mut(child).expect("child not in tree");
            child_node.previous_sibling = Some(last_child);
        } else {
            let mut parent_node = self.get_node_mut(parent).expect("parent to be part of the tree");
            parent_node.first_child = Some(child);
        }
    }
    pub fn set_parent(&mut self, parent: TreeIndex, child: TreeIndex) {
        let node = self.get_node_mut(child).expect("Node not in tree");
        node.parent = Some(parent);
    }
    pub fn get_node(&self, index: TreeIndex) -> Option<&MutableTreeNode> {
        return if let Some(node) = self.nodes.get(index) {
            node.as_ref()
        } else {
            None
        };
    }
    fn get_node_mut(&mut self, index: TreeIndex) -> Option<&mut MutableTreeNode> {
        return if let Some(node) = self.nodes.get_mut(index) {
            node.as_mut()
        } else {
            None
        };
    }
    pub fn get_children(&self, index: TreeIndex) -> Vec<TreeIndex> {
        let mut children = Vec::new();
        let mut node = self.get_node(index).expect("Node not in tree");
        if let Some(fist_child) = node.first_child {
            children.push(fist_child);
            let mut child = self.get_node(fist_child).expect("Node not in tree");
            if let Some(sibling) = child.next_sibling {
                children.push(sibling);
                child = self.get_node(sibling).expect("expected sibling node to be in the tree");
            }
        }
        return children;
    }
    pub fn set_length(&mut self, index: TreeIndex, bl: f64) {
        let node = self.get_node_mut(index).expect("node not in tree");
        node.length = Some(bl);
    }
    pub fn get_annotation(&self, index: TreeIndex, key: &str)->Option<&AnnotationValue> {
        return if let Some(annotation) = self.node_annotations[index].as_ref() {
            annotation.get(key)
        } else {
            None
        }
    }
    pub fn get_root(&self)->Option<TreeIndex>{
        self.root
    }
    pub fn annotate_node(&mut self, index: TreeIndex, key: String, value: AnnotationValue) {
        let possible_annotation = self.node_annotations[index].as_mut();
        if let Some(annotation)=possible_annotation{
                annotation.insert(key, value);
        }else{
           println!("node not found");
        }
    }
    pub fn label_node(&mut self, index: TreeIndex, label: String) {
        let mut node = self.get_node_mut(index);
        if let Some(n) = node{
            n.label = Some(label);
        }else{
            println!("node not found");
        }
    }
}

pub struct PreorderIter {
    stack: Vec<TreeIndex>
}

impl PreorderIter {
    pub fn new(root: Option<TreeIndex>) -> Self {
        if let Some(index) = root {
            PreorderIter {
                stack: vec![index]
            }
        } else {
            PreorderIter { stack: vec![] }
        }
    }
    pub fn next(&mut self, tree: &MutableTree) -> Option<TreeIndex> {
        while let Some(node_index) = self.stack.pop() {
            if let Some(node) = tree.get_node(node_index) {
                if let Some(sibling) = node.next_sibling {
                    self.stack.push(sibling);
                }
                if let Some(child) = node.first_child {
                    self.stack.push(child);
                }
                return Some(node_index);
            }
        }
        return None;
    }
}



