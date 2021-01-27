use std::option::Option;
use super::fixed_tree::FixedNode;
use std::collections::{HashMap, HashSet};
use crate::parsers::newick_parser::AnnotationValue;
use std::collections::hash_map::Keys;

pub type TreeIndex = usize;


pub struct Taxon {
    name: Option<String>,
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
    pub height: Option<f64>,
    pub annotations: HashMap<String, AnnotationValue>,
    number: usize,

}

impl MutableTreeNode {
    pub(crate) fn new(taxon: Option<String>,
                      number: usize) -> Self {
        MutableTreeNode { taxon: taxon, label: None, parent: None, first_child: None, next_sibling: None, previous_sibling: None, length: None, height: None, annotations: HashMap::new(), number }
    }
}

pub struct MutableTree {
    nodes: Vec<MutableTreeNode>,
    external_nodes: Vec<TreeIndex>,
    internal_nodes: Vec<TreeIndex>,
    annotation_type: HashMap<String, AnnotationValue>,
    taxon_node_map: HashMap<String, TreeIndex>,
    root: Option<TreeIndex>,
    heights_known: bool,
    branchlengths_known: bool,

}

impl MutableTree {
    pub fn from_fixed_node(root: FixedNode) -> Self {
        let mut tree = MutableTree {
            nodes: Vec::new(),
            external_nodes: Vec::new(),
            internal_nodes: Vec::new(),
            annotation_type: HashMap::new(),
            taxon_node_map: HashMap::new(),
            root: None,
            heights_known: false,
            branchlengths_known: false,
        };
        tree.fixed_node_helper(root, None);
        tree.set_root(Some(0));
        return tree;
    }

    fn fixed_node_helper(&mut self, node: FixedNode, parent: Option<TreeIndex>) {
        let index = self.nodes.len();
        let new_node = MutableTreeNode::new(node.taxon.clone(), index);
        self.nodes.push(new_node);

        if let Some(length) = node.length {
            self.set_length(&index, length);
        }

        if let Some(mut annotation_map) = node.annotations {
            for (key, value) in annotation_map.into_iter() {
                self.annotate_node(&index, key, value);
            }
        }

        if node.children.len() > 0 {
            self.internal_nodes.push(index);
        } else {
            self.external_nodes.push(index);
            if let Some(taxon) = node.taxon.clone() {
                self.taxon_node_map.insert(taxon.clone(), index);
            }
        }
        if let Some(p) = parent {
            self.add_child(&p, &index);
            self.set_parent(&p, &index);
        }
        for child in node.children.into_iter() {
            self.fixed_node_helper(*child, Some(index));
        }
    }

    pub fn from_tree(tree: &mut MutableTree, taxa: &HashSet<String>) -> Self {
        let mut me = MutableTree {
            nodes: Vec::new(),
            external_nodes: Vec::new(),
            internal_nodes: Vec::new(),
            annotation_type: HashMap::new(),
            taxon_node_map: HashMap::new(),
            root: None,
            heights_known: false,
            branchlengths_known: false,
        };
        let root = tree.get_root().expect("every tree should have a root at least nominally");
        me.tree_helper(tree, root, taxa);
        me
    }

    fn tree_helper(&mut self, tree: &mut MutableTree, node: TreeIndex, taxa: &HashSet<String>) -> Option<TreeIndex> {
        let mut new_node = None;
        if tree.get_num_children(&node) == 0 {
            //make external node
            if let Some(taxon) = &tree.get_unwrapped_node(&node).taxon {
                new_node = self.make_external_node(taxon, &taxa);
            }
        } else {
            let nchildren = tree.get_num_children(&node);
            let mut children: Vec<usize> = vec![];
            let mut visited = 0;
            while visited < nchildren {
                let child = tree.get_child(&node, visited);
                if let Some(child_node) = child {
                    let new_child = self.tree_helper(tree, child_node, taxa);
                    if let Some(new_child_index) = new_child {
                        children.push(new_child_index)
                    }
                } else {
                    panic!("aaaaah!")
                }
                visited += 1;
            }
            if children.len() > 1 {
                new_node = Some(self.make_internal_node(children));
            } else {
                if children.len() == 1 {
                    return Some(children.remove(0));
                }
            }
        }
        if let Some(new_node_i) = new_node {
            self.set_height(&new_node_i, tree.get_height(&node));
            // copy annotations
            let annotation_map = &tree.get_unwrapped_node(&node).annotations;
            for (key, value) in annotation_map.into_iter() {
                self.annotate_node(&new_node_i, key.clone(), value.clone());
            }
        }
        new_node
    }
    pub fn get_height(&mut self, node: &TreeIndex) -> f64 {
        if !self.heights_known {
            self.calc_node_heights();
        }
        self.get_unwrapped_node(node).height.expect("how did it come to this")
    }
    fn get_current_height(&self, node: TreeIndex) -> f64 {
        self.get_unwrapped_node(&node).height.expect("how did it come to this. I thought heights were trust worthy")
    }
    fn calc_node_heights(&mut self) {
        self.heights_known = true;
        self.calc_height_above_root();
        let mut rtt = 0.0;
        for node_ref in self.external_nodes.iter() {
            let h = self.get_current_height(*node_ref);
            if h > rtt {
                rtt = h;
            }
        }

        let mut i = 0;
        while i < self.nodes.len() {
            let height = rtt - self.get_current_height(i);
            self.set_height(&i, height);
            i += 1;
        }
    }
    fn calc_height_above_root(&mut self) {
        let mut preorder = self.preorder_iter();
        for node_ref in preorder {
            if let Some(p) = self.get_parent(&node_ref) {
                let l = self.get_length(&node_ref).expect("tree needs lengths to get heights");
                let pheight = self.get_current_height(p);
                self.set_height(&node_ref, l + pheight);
            } else {
                //at root
                self.set_height(&node_ref, 0.0);
            }
        }
        println!("{:?}", self.nodes)
    }


    fn set_height(&mut self, node_ref: &TreeIndex, height: f64) {
        let node = self.get_unwrapped_node_mut(&node_ref);
        node.height = Some(height);
        self.branchlengths_known = false;
    }

    fn make_external_node(&mut self, taxon: &String, taxa: &HashSet<String>) -> Option<TreeIndex> {
        if taxa.contains(taxon) {
            let index = self.nodes.len();
            let new_node = MutableTreeNode::new(Some(taxon.clone()), index);
            self.nodes.push(new_node);
            self.external_nodes.push(index);
            self.taxon_node_map.insert(taxon.clone(), index);

            Some(index)
        } else {
            None
        }
    }

    fn make_internal_node(&mut self, children: Vec<TreeIndex>) -> TreeIndex {
        let index = self.nodes.len();
        let new_node = MutableTreeNode::new(None, index);
        self.nodes.push(new_node);
        self.internal_nodes.push(index);
        for child in children.into_iter() {
            self.add_child(&index, &child);
            self.set_parent(&index, &child)
        }
        self.set_root(Some(index));
        index
    }


    pub fn preorder_iter(&self) -> PreOrderIterator {
        PreOrderIterator::new(self.root, self)
    }
    pub fn set_root(&mut self, root: Option<TreeIndex>) {
        self.root = root
    }

    pub fn add_child(&mut self, parent: &TreeIndex, child: &TreeIndex) {
        let parent_node = self.get_node_mut(*parent).expect("Node not in tree");
        if let Some(first_child) = parent_node.first_child {
            let mut sibling_node = self.get_node_mut(first_child).expect("Node not in tree");
            while let Some(sibling) = sibling_node.next_sibling {
                sibling_node = self.get_node_mut(sibling).expect("expected sibling node to be in the tree");
            }
            sibling_node.next_sibling = Some(*child);
            let previous_sib_i = sibling_node.number;
            let mut child_node = self.get_node_mut(*child).expect("node in tree");
            child_node.previous_sibling = Some(previous_sib_i);
        } else {
            let mut parent_node = self.get_node_mut(*parent).expect("parent to be part of the tree");
            parent_node.first_child = Some(*child);
        }
    }

    pub fn remove_child(&mut self, parent: TreeIndex, child: TreeIndex) {
        let mut successful_removal = true;
        let child_node = self.get_unwrapped_node(&child);
        if let Some(previous_slibling_i) = child_node.previous_sibling {
            if let Some(next_sibling_i) = child_node.next_sibling {
                let mut prev_sib = self.get_unwrapped_node_mut(&previous_slibling_i);
                prev_sib.next_sibling = Some(next_sibling_i);
                let mut next_sib = self.get_unwrapped_node_mut(&next_sibling_i);
                next_sib.previous_sibling = Some(previous_slibling_i);
            } else {
                let mut prev_sib = self.get_unwrapped_node_mut(&previous_slibling_i);
                prev_sib.next_sibling = None;
            }
        } else {
            let parent_node = self.get_unwrapped_node_mut(&parent);
            if let Some(first_child) = parent_node.first_child {
                if first_child == child {
                    let child_node = self.get_unwrapped_node(&child);
                    if let Some(next_sibling_i) = child_node.next_sibling {
                        let parent_node = self.get_unwrapped_node_mut(&parent);
                        parent_node.first_child = Some(next_sibling_i);
                        let mut next_sib = self.get_unwrapped_node_mut(&next_sibling_i);
                        next_sib.previous_sibling = None;
                    } else {
                        let parent_node = self.get_unwrapped_node_mut(&parent);
                        parent_node.first_child = None;
                    }
                } else {
                    println!("not a child of the parent node! TODO make this a warning");
                    successful_removal = false;
                }
            }
        }

        if successful_removal {
            let mut_child = self.get_unwrapped_node_mut(&child);
            mut_child.next_sibling = None;
            mut_child.previous_sibling = None;
        }
    }

    pub fn set_parent(&mut self, parent: &TreeIndex, child: &TreeIndex) {
        let node = self.get_node_mut(*child).expect("Node not in tree");
        node.parent = Some(*parent);
    }
    pub fn get_node(&self, index: TreeIndex) -> Option<&MutableTreeNode> {
       self.nodes.get(index)
    }
    //TODO should index be borrowed or owned here?
    fn get_node_mut(&mut self, index: TreeIndex) -> Option<&mut MutableTreeNode> {
      self.nodes.get_mut(index)
    }
    fn get_unwrapped_node(&self, index: &TreeIndex) -> &MutableTreeNode {
        return self.get_node(*index).expect("node not in tree");
    }
    fn get_unwrapped_node_mut(&mut self, index: &TreeIndex) -> &mut MutableTreeNode {
        return self.get_node_mut(*index).expect("node not in tree");
    }

    pub fn get_node_count(&self) -> usize {
        self.nodes.len()
    }
    pub fn get_internal_node_count(&self) -> usize {
        self.internal_nodes.len()
    }
    pub fn get_external_node_count(&self) -> usize {
        self.external_nodes.len()
    }

    pub fn get_num_children(&self, node_ref: &TreeIndex) -> TreeIndex {
        let mut count = 0;
        let parent_node = self.get_unwrapped_node(&node_ref);
        if let Some(first_child) = parent_node.first_child {
            let mut sibling_node = self.get_unwrapped_node(&first_child);
            loop {
                if let Some(sibling) = sibling_node.next_sibling {
                    sibling_node = self.get_unwrapped_node(&sibling);
                    count += 1;
                } else {
                    break;
                }
            }
        }
        return count;
    }
    pub fn get_child(&self, node_ref: &TreeIndex, index: usize) -> Option<TreeIndex> {
        let mut count = 0;
        let parent_node = self.get_unwrapped_node(&node_ref);
        if let Some(first_child) = parent_node.first_child {
            if index == 0 {
                return Some(first_child);
            }
            let mut sibling_node = self.get_unwrapped_node(&first_child);
            loop {
                if let Some(sibling) = sibling_node.next_sibling {
                    sibling_node = self.get_unwrapped_node(&sibling);
                    count += 1;
                    if count == index {
                        return Some(sibling);
                    }
                } else {
                    break;
                }
            }
        }
        return None;
    }
    pub fn get_next_sibling(&self, index: &TreeIndex) -> Option<TreeIndex> {
        let node = self.get_unwrapped_node(index);
        if let Some(sib) = node.next_sibling {
            return Some(sib);
        }
        return None;
    }
    pub fn get_previous_sibling(&self, index: &TreeIndex) -> Option<TreeIndex> {
        let node = self.get_unwrapped_node(index);
        if let Some(sib) = node.previous_sibling {
            return Some(sib);
        }
        return None;
    }
    pub fn get_parent(&self, index: &TreeIndex) -> Option<TreeIndex> {
        let node = self.get_unwrapped_node(index);
        if let Some(p) = node.parent {
            return Some(p);
        }
        return None;
    }
    pub fn set_length(&mut self, index: &TreeIndex, bl: f64) {
        let node = self.get_node_mut(*index).expect("node not in tree");
        node.length = Some(bl);
    }
    pub fn get_length(&self, node_ref: &TreeIndex) -> Option<f64> {
        let node = self.get_node(*node_ref).expect("node not in tree");
        if let Some(l) = node.length {
            Some(l)
        } else {
            None
        }
    }
    pub fn get_annotation(&self, index: &TreeIndex, key: &str) -> Option<&AnnotationValue> {
        return self.get_unwrapped_node(index).annotations.get(key);
    }
    pub fn get_annotation_keys(&self) -> Keys<'_, String, AnnotationValue> {
        self.annotation_type.keys()
    }
    pub fn get_root(&self) -> Option<TreeIndex> {
        self.root
    }
    pub fn annotate_node(&mut self, index: &TreeIndex, key: String, value: AnnotationValue) {
        if let Some(annotation) = self.annotation_type.get(&key) {
            let value_type = std::mem::discriminant(&value);
            let annotation_type = std::mem::discriminant(annotation);
            if value_type == annotation_type {
                let mut node = self.get_unwrapped_node_mut(index);
                node.annotations.insert(key, value);
            } else {
                panic!("tried to annotate node with an missmatched annotation type");
            }
        } else {
            match value {
                AnnotationValue::Discrete(_) => {
                    self.annotation_type.insert(key.clone(), AnnotationValue::Discrete("".to_string()));
                }
                AnnotationValue::Continuous(_) => {
                    self.annotation_type.insert(key.clone(), AnnotationValue::Continuous(0.0));
                }
                AnnotationValue::Set(_) => {
                    //TODO check internal types
                    self.annotation_type.insert(key.clone(), AnnotationValue::Set(vec![AnnotationValue::Discrete("0".to_string())]));
                }
            }
            let mut node = self.get_unwrapped_node_mut(index);
            node.annotations.insert(key, value);
        }
    }
    pub fn label_node(&mut self, index: &TreeIndex, label: String) {
        let node = self.get_unwrapped_node_mut(&index);
        node.label = Some(label);
    }

    pub fn get_label(&self, index: &TreeIndex) -> &Option<String> {
        let node = self.get_unwrapped_node(index);
        &node.label
    }
    pub fn get_taxon_node(&self, taxon: &String) -> Option<usize> {
        if let Some(i)=self.taxon_node_map.get(taxon){
            return Some(*i)
        }else{
            return None
        }
    }
}

pub struct PreOrderIterator {
    stack: Vec<TreeIndex>
}

impl PreOrderIterator {
    pub fn new(root: Option<TreeIndex>, tree: &MutableTree) -> Self {
        let mut local_stack = vec![];
        if let Some(index) = root {
            local_stack = vec![index];
        } else {
            let root = tree.get_root().expect("each tree should have a root");
            local_stack = vec![root];
        }
        let mut me = PreOrderIterator {
            stack: vec![]
        };

        while let Some(node_index) = local_stack.pop() {
            if let Some(node) = tree.get_node(node_index) {
                if let Some(sibling) = node.next_sibling {
                    local_stack.push(sibling);
                }
                if let Some(child) = node.first_child {
                    local_stack.push(child);
                }
                me.stack.push(node_index)
            }
        }
        me
    }
}

impl Iterator for PreOrderIterator {
    type Item = TreeIndex;

    fn next(&mut self) -> Option<Self::Item> {
        if self.stack.len() > 0 {
            Some(self.stack.remove(0))
        } else {
            None
        }
    }
}

impl DoubleEndedIterator for PreOrderIterator {
    fn next_back (&mut self )-> Option< Self::Item> {
        self.stack.pop()
    }
}


