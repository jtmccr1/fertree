use super::fixed_tree::FixedNode;
use super::AnnotationValue;
use std::collections::hash_map::Keys;
use std::collections::{HashMap, HashSet};
use std::option::Option;
use std::cmp::Ordering;

pub type TreeIndex = usize;

//TODO think more about missing data. Should these be options? Should they be guaranteed
//TODO add tree annotation
//TODO adopt nodeorder
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
    pub number: usize,
}

impl MutableTreeNode {
    pub(crate) fn new(taxon: Option<String>, number: usize) -> Self {
        MutableTreeNode {
            taxon,
            label: None,
            parent: None,
            first_child: None,
            next_sibling: None,
            previous_sibling: None,
            length: None,
            height: None,
            annotations: HashMap::new(),
            number,
        }
    }
}

#[derive(Debug)]
pub struct MutableTree {
    pub nodes: Vec<MutableTreeNode>,
    pub external_nodes: Vec<TreeIndex>,
    pub internal_nodes: Vec<TreeIndex>,
    pub annotation_type: HashMap<String, AnnotationValue>,
    pub taxon_node_map: HashMap<String, TreeIndex>,
    pub label_node_map: HashMap<String, TreeIndex>,
    pub root: Option<TreeIndex>,
    pub heights_known: bool,
    pub branchlengths_known: bool,
    pub id: Option<String>,
    pub tree_annotation: HashMap<String, AnnotationValue>,
}

impl Default for MutableTree {
    fn default() -> Self {
        Self::new()
    }
}

impl MutableTree {
    pub fn new() -> Self {
        MutableTree {
            nodes: vec![],
            external_nodes: vec![],
            internal_nodes: vec![],
            annotation_type: Default::default(),
            taxon_node_map: Default::default(),
            label_node_map:Default::default(),
            root: None,
            heights_known: false,
            branchlengths_known: false,
            id: None,
            tree_annotation: HashMap::new(),
        }
    }
    pub fn from_fixed_node(root: FixedNode) -> Self {
        let mut tree = MutableTree::new();
        tree.branchlengths_known = true;
        tree.fixed_node_helper(root, None);
        tree.set_root(Some(0));
        tree.calc_node_heights();
        tree.branchlengths_known = true;
        tree
    }

    fn fixed_node_helper(&mut self, node: FixedNode, parent: Option<TreeIndex>) {
        let index = self.nodes.len();
        let new_node = MutableTreeNode::new(node.taxon.clone(), index);
        self.nodes.push(new_node);

        if let Some(length) = node.length {
            self.set_length(index, length);
        }
        if let Some(label) = node.label {
            self.set_label(index, label);
        }

        if let Some(annotation_map) = node.annotations {
            for (key, value) in annotation_map.into_iter() {
                self.annotate_node(index, key, value);
            }
        }

        if !node.children.is_empty() {
            self.internal_nodes.push(index);
        } else {
            self.external_nodes.push(index);
            if let Some(taxon) = node.taxon {
                self.label_node_map.insert(taxon.clone(), index);
                self.taxon_node_map.insert(taxon, index);
            }
        }
        if let Some(p) = parent {
            self.add_child(p, index);
            self.set_parent(p, index);
        }
        for child in node.children.into_iter() {
            self.fixed_node_helper(child, Some(index));
        }
    }

    pub fn from_tree(tree: &MutableTree, taxa: &HashSet<String>) -> Self {
        let mut me = MutableTree::new();
        let root = tree
            .get_root()
            .expect("every tree should have a root at least nominally");
        me.tree_helper(tree, root, taxa);
        me.heights_known = true;
        me.calculate_branchlengths();
        me
    }

    pub fn copy_subtree(tree: &MutableTree, node: TreeIndex, taxa: &HashSet<String>) -> Self {
        let mut me = MutableTree::new();
        me.tree_helper(tree, node, taxa);
        me.heights_known = true;
        trace!("in tree: {}",me.branchlengths_known);
        trace!("in tree: {}",me.heights_known);
        me
    }
    fn tree_helper(
        &mut self,
        tree: &MutableTree,
        node: TreeIndex,
        taxa: &HashSet<String>,
    ) -> Option<TreeIndex> {
        let mut new_node = None;
        if tree.get_num_children(node) == 0 {
            //make external node
            if let Some(taxon) = &tree.get_unwrapped_node(node).taxon {
                new_node = self.make_external_node(taxon, Some(&taxa));
                self.set_height(
                    new_node.unwrap(),
                    tree.get_height(node)
                        .expect("found a node without a height"),
                );
                // copy annotations
                let annotation_map = &tree.get_unwrapped_node(node).annotations;
                for (key, value) in annotation_map.iter() {
                    self.annotate_node(new_node.unwrap(), key.clone(), value.clone());
                }
            }
            new_node
        } else {
            let nchildren = tree.get_num_children(node);
            let mut children: Vec<usize> = vec![];
            let mut visited = 0;
            while visited < nchildren {
                let child = tree.get_child(node, visited);
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
            match children.len().cmp(&1){
                Ordering::Less => {panic!("should have caught this error before this line!")}
                Ordering::Equal => { Some(children[0])}
                Ordering::Greater => {
                    new_node = Some(self.make_root_node(children));
                    self.set_height(
                        new_node.unwrap(),
                        tree.get_height(node)
                            .expect("found a node without a height"),
                    );
                    // copy annotations
                    let annotation_map = &tree.get_unwrapped_node(node).annotations;
                    for (key, value) in annotation_map.iter() {
                        self.annotate_node(new_node.unwrap(), key.clone(), value.clone());
                    }
                    new_node}
            }
        }
    }
    pub fn get_height(&self, node: TreeIndex) -> Option<f64> {
        self.get_unwrapped_node(node).height
    }
    pub fn get_node_label(&self, node: TreeIndex) -> &Option<String> {
        &self.get_unwrapped_node(node).label
    }
    // fn get_height_safely(&mut self, node: TreeIndex) -> f64 {
    //     if !self.heights_known {
    //         self.calc_node_heights();
    //     }
    //     self.get_unwrapped_node(node)
    //         .height
    //         .expect("how did it come to this. I thought heights were trust worthy")
    // }
    //TODO clean up bl/h knowledge;
    pub fn calc_node_heights(&mut self) {
        if !self.heights_known {
            self.heights_known = true;
            self.calc_height_above_root();
            let mut rtt = 0.0;
            for node_ref in self.external_nodes.iter() {
                let h = self.get_height(*node_ref).unwrap();
                if h > rtt {
                    rtt = h;
                }
            }

            let mut i = 0;
            while i < self.nodes.len() {
                let height = rtt - self.get_height(i).unwrap();
                self.set_height(i, height);
                i += 1;
            }
        }
    }

    pub fn calc_relative_node_heights(&mut self,origin:f64) {
            self.heights_known = true;
            self.calc_height_above_root();
            let mut rtt = 0.0;
            for node_ref in self.external_nodes.iter() {
                let h = self.get_height(*node_ref).unwrap();
                if h > rtt {
                    rtt = h;
                }
            }

            let mut i = 0;
            while i < self.nodes.len() {
                let height = origin-(rtt - self.get_height(i).unwrap());
                self.set_height(i, height);
                i += 1;
            }

    }

    pub fn calculate_branchlengths(&mut self) {
       if  !self.branchlengths_known {

        self.branchlengths_known = true;
        let mut i = 0;
        while i < self.nodes.len() {
            if i != self.root.expect("how is this tree not rooted") {
                let length = self
                    .get_height(
                        self.get_parent(i)
                            .expect("node does not have a parent in tree"),
                    )
                    .expect("parent node should have height")
                    - self.get_height(i).expect("node should have height");
                self.set_length(i, length);
            }
            i += 1;
        }
    }
    }
    fn calc_height_above_root(&mut self) {
        let preorder = self.preorder_iter();
        for node_ref in preorder {
            if let Some(p) = self.get_parent(node_ref) {
                let l = self
                    .get_length(node_ref)
                    .expect(&*format!("no length on node {}", node_ref));
                let pheight = self.get_height(p).unwrap();
                self.set_height(node_ref, l + pheight);
            } else {
                //at root
                self.set_height(node_ref, 0.0);
            }
        }
    }

    pub fn set_height(&mut self, node_ref: TreeIndex, height: f64) {
        let node = self.get_unwrapped_node_mut(node_ref);
        node.height = Some(height);
        self.branchlengths_known = false;
    }

    pub fn make_external_node(&mut self, taxon: &str, taxa_set: Option<&HashSet<String>>) -> Option<TreeIndex> {
        if let Some(taxa) = taxa_set {
            if taxa.contains(taxon) {
                let index = self.nodes.len();
                let new_node = MutableTreeNode::new(Some(taxon.to_string()), index);
                self.nodes.push(new_node);
                self.external_nodes.push(index);
                self.taxon_node_map.insert(taxon.to_string(), index);
                self.label_node_map.insert(taxon.to_string(), index);

                Some(index)
            } else {
                None
            }
        } else {
            let index = self.nodes.len();
            let new_node = MutableTreeNode::new(Some(taxon.to_string()), index);
            self.nodes.push(new_node);
            self.external_nodes.push(index);
            self.taxon_node_map.insert(taxon.to_string(), index);
            self.label_node_map.insert(taxon.to_string(), index);

            Some(index)
        }
    }
    /// Make and return an internal node. Provided children will be added to the node in
    /// the order they appear in the input vector;
    pub fn make_internal_node(&mut self, children: Vec<TreeIndex>) -> TreeIndex {
        let index = self.nodes.len();
        let new_node = MutableTreeNode::new(None, index);
        self.nodes.push(new_node);
        self.internal_nodes.push(index);
        for child in children.into_iter() {
            let child_node = self.get_node_mut(child).unwrap();
            child_node.next_sibling = None;
            child_node.previous_sibling = None;
            self.add_child(index, child);
            self.set_parent(index, child)
        }
        index
    }
    fn make_root_node(&mut self, children: Vec<TreeIndex>) -> TreeIndex {
        let index = self.make_internal_node(children);
        self.set_root(Some(index));
        index
    }

    pub fn preorder_iter(&self) -> PreOrderIterator {
        PreOrderIterator::new(self.root, self)
    }
    pub fn set_root(&mut self, root: Option<TreeIndex>) {
        self.root = root
    }

    pub fn add_child(&mut self, parent: TreeIndex, child: TreeIndex) {
        let parent_node = self.get_node_mut(parent).expect("Node not in tree");
        if let Some(first_child) = parent_node.first_child {
            let mut sibling_node = self.get_node_mut(first_child).expect("Node not in tree");
            while let Some(sibling) = sibling_node.next_sibling {
                sibling_node = self
                    .get_node_mut(sibling)
                    .expect("expected sibling node to be in the tree");
            }
            sibling_node.next_sibling = Some(child);
            let previous_sib_i = sibling_node.number;
            let mut child_node = self.get_node_mut(child).expect("node not in tree");
            child_node.previous_sibling = Some(previous_sib_i);
        } else {
            let mut parent_node = self
                .get_node_mut(parent)
                .expect("parent to be part of the tree");
            parent_node.first_child = Some(child);
        }
    }

    pub fn remove_child(&mut self, parent: TreeIndex, child: TreeIndex) -> Option<TreeIndex> {
        let mut successful_removal = true;
        let child_node = self.get_unwrapped_node(child);
        if let Some(previous_slibling_i) = child_node.previous_sibling {
            if let Some(next_sibling_i) = child_node.next_sibling {
                let mut prev_sib = self.get_unwrapped_node_mut(previous_slibling_i);
                prev_sib.next_sibling = Some(next_sibling_i);
                let mut next_sib = self.get_unwrapped_node_mut(next_sibling_i);
                next_sib.previous_sibling = Some(previous_slibling_i);
            } else {
                let mut prev_sib = self.get_unwrapped_node_mut(previous_slibling_i);
                prev_sib.next_sibling = None;
            }
        } else {
            let parent_node = self.get_unwrapped_node_mut(parent);
            if let Some(first_child) = parent_node.first_child {
                if first_child == child {
                    let child_node = self.get_unwrapped_node(child);
                    if let Some(next_sibling_i) = child_node.next_sibling {
                        let parent_node = self.get_unwrapped_node_mut(parent);
                        parent_node.first_child = Some(next_sibling_i);
                        let mut next_sib = self.get_unwrapped_node_mut(next_sibling_i);
                        next_sib.previous_sibling = None;
                    } else {
                        let parent_node = self.get_unwrapped_node_mut(parent);
                        parent_node.first_child = None;
                    }
                } else {
                    println!("not a child of the parent node! TODO make this a warning");
                    successful_removal = false;
                }
            }
        }

        if successful_removal {
            let mut_child = self.get_unwrapped_node_mut(child);
            mut_child.next_sibling = None;
            mut_child.previous_sibling = None;
            Some(child)
        } else {
            None
        }
    }

    pub fn set_parent(&mut self, parent: TreeIndex, child: TreeIndex) {
        let node = self.get_node_mut(child).expect("Node not in tree");
        node.parent = Some(parent);
    }
    pub fn get_node(&self, index: TreeIndex) -> Option<&MutableTreeNode> {
        self.nodes.get(index)
    }
    fn get_node_mut(&mut self, index: TreeIndex) -> Option<&mut MutableTreeNode> {
        self.nodes.get_mut(index)
    }
    fn get_unwrapped_node(&self, index: TreeIndex) -> &MutableTreeNode {
        return self
            .get_node(index)
            .expect(&*format!("node {} not in tree", index));
    }
    fn get_unwrapped_node_mut(&mut self, index: TreeIndex) -> &mut MutableTreeNode {
        return self
            .get_node_mut(index)
            .expect(&*format!("node {} not in tree", index));
    }

    pub fn get_taxon(&self, node_ref: TreeIndex) -> Option<&str> {
        let node = self.get_unwrapped_node(node_ref);
        node.taxon.as_deref()
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

    pub fn get_num_children(&self, node_ref: TreeIndex) -> TreeIndex {
        let mut count = 0;
        let parent_node = self.get_unwrapped_node(node_ref);
        if let Some(first_child) = parent_node.first_child {
            count += 1;
            let mut sibling_node = self.get_unwrapped_node(first_child);
            while let Some(sibling) = sibling_node.next_sibling {
                sibling_node = self.get_unwrapped_node(sibling);
                count += 1;
            }
        }
        count
    }
    pub fn get_child(&self, node_ref: TreeIndex, index: usize) -> Option<TreeIndex> {
        let mut count = 0;
        let parent_node = self.get_unwrapped_node(node_ref);
        if let Some(first_child) = parent_node.first_child {
            if index == 0 {
                return Some(first_child);
            }
            let mut sibling_node = self.get_unwrapped_node(first_child);
            while let Some(sibling) = sibling_node.next_sibling {
                sibling_node = self.get_unwrapped_node(sibling);
                count += 1;
                if count == index {
                    return Some(sibling);
                }
            }
        }
        None
    }
    pub fn get_external_node(&self, index: usize) -> Option<&MutableTreeNode> {
        self.nodes.get(self.external_nodes[index])
    }
    pub fn get_internal_node(&self, index: usize) -> Option<&MutableTreeNode> {
        self.nodes.get(self.internal_nodes[index])
    }

    pub fn get_children(&self, node: TreeIndex) -> Vec<TreeIndex> {
        let mut kids: Vec<TreeIndex> = vec![];
        let parent_node = self.get_unwrapped_node(node);
        if let Some(first_child) = parent_node.first_child {
            kids.push(first_child);
            let mut sibling_node = self.get_unwrapped_node(first_child);
            while let Some(sibling) = sibling_node.next_sibling {
                kids.push(sibling);
                sibling_node = self.get_unwrapped_node(sibling);
            }
        }
        kids
    }
    pub fn get_next_sibling(&self, index: TreeIndex) -> &Option<TreeIndex> {
        let node = self.get_unwrapped_node(index);
        &node.next_sibling
    }
    pub fn get_previous_sibling(&self, index: TreeIndex) -> &Option<TreeIndex> {
        let node = self.get_unwrapped_node(index);
        &node.previous_sibling
    }
    pub fn get_first_child(&self, index: TreeIndex) -> &Option<TreeIndex> {
        let node = self.get_unwrapped_node(index);
        &node.first_child
    }
    pub fn get_parent(&self, index: TreeIndex) -> Option<TreeIndex> {
        let node = self.get_unwrapped_node(index);
        if let Some(p) = node.parent {
            return Some(p);
        }
        None
    }
    pub fn set_label(&mut self, index: TreeIndex, label: String) {
        let node = self.get_node_mut(index).expect("node not in tree");
        node.label = Some(label.clone());
        self.label_node_map.insert(label,index);
    }

    pub fn set_length(&mut self, index: TreeIndex, bl: f64) {
        self.heights_known = false;
        let node = self.get_node_mut(index).expect("node not in tree");
        node.length = Some(bl);
    }
    pub fn get_length(&self, node_ref: TreeIndex) -> Option<f64> {
        let node = self.get_node(node_ref).expect("node not in tree");
        node.length
    }
    pub fn get_annotation(&self, index: TreeIndex, key: &str) -> Option<&AnnotationValue> {
        return self.get_unwrapped_node(index).annotations.get(key);
    }
    //TODO public members or getter/setter?
    pub fn get_annotation_keys(&self) -> Keys<'_, String, AnnotationValue> {
        self.annotation_type.keys()
    }
    pub fn is_external(&self, node_ref: TreeIndex) -> bool {
        self.get_unwrapped_node(node_ref).first_child.is_none()
    }
    pub fn get_root(&self) -> Option<TreeIndex> {
        self.root
    }
    pub fn annotate_node(&mut self, index: TreeIndex, key: String, value: AnnotationValue) {
        if let Some(annotation) = self.annotation_type.get(&key) {
            let value_type = std::mem::discriminant(&value);
            let annotation_type = std::mem::discriminant(annotation);

            if value_type == annotation_type {
            let node = self.get_unwrapped_node_mut(index);
                node.annotations.insert(key, value);
            } else if let AnnotationValue::Continuous(c)=value{
                    warn!("coercing {} to string for annotation {}",c,key.as_str());
                    self.annotate_node(index,key,AnnotationValue::Discrete(c.to_string())) ;
            }else{
                panic!("tried to annotate node with an missmatched annotation type for {}, found {} expected {}",key.as_str(),&value,&annotation);
            }
        } else {
            match value {
                AnnotationValue::Discrete(_) => {
                    self.annotation_type
                        .insert(key.clone(), AnnotationValue::Discrete("".to_string()));
                }
                AnnotationValue::Continuous(_) => {
                    self.annotation_type
                        .insert(key.clone(), AnnotationValue::Continuous(0.0));
                }
                AnnotationValue::Boolean(_) => {
                    self.annotation_type
                        .insert(key.clone(), AnnotationValue::Boolean(true));
                }
                AnnotationValue::Set(_) => {
                    //TODO check internal types
                    self.annotation_type.insert(
                        key.clone(),
                        AnnotationValue::Set(vec![AnnotationValue::Discrete("0".to_string())]),
                    );
                },
                AnnotationValue::MarkovJump(_) => {
                    panic!("Markov jumps must be in sets of annotations not single.");
                }
            }
            let node = self.get_unwrapped_node_mut(index);
            node.annotations.insert(key, value);
        }
    }
    pub fn label_node(&mut self, index: TreeIndex, label: String) {
        let node = self.get_unwrapped_node_mut(index);
        node.label = Some(label.clone());
        self.label_node_map.insert(label, index);
    }

    pub fn get_label(&self, index: TreeIndex) -> Option<&str> {
        let node = self.get_unwrapped_node(index);
        node.label.as_deref()
    }
    pub fn get_taxon_node(&self, taxon: &str) -> Option<usize> {
        self.taxon_node_map.get(taxon).copied()
    }
    pub fn get_label_node(&self, label: &str) -> Option<usize> {
        self.label_node_map.get(label).copied()
    }
    pub fn annotate_tree(&mut self, key: String, value: AnnotationValue) {
        self.tree_annotation.insert(key, value);
    }
    pub fn get_tree_annnotation(&mut self, key: &str) -> Option<&AnnotationValue> {
        self.tree_annotation.get(key)
    }

    pub fn set_id(&mut self, id: String) {
        self.id = Some(id);
    }
    pub fn get_id(&self) ->Option<&str>{
        self.id.as_deref()
    }
}
//TODO I don't like that this is not lazy

pub struct PreOrderIterator {
    stack: Vec<TreeIndex>,
}

impl PreOrderIterator {
    pub fn new(root: Option<TreeIndex>, tree: &MutableTree) -> Self {
        let mut local_stack: Vec<TreeIndex>;
        if let Some(index) = root {
            local_stack = vec![index];
        } else {
            let root = tree.get_root().expect("each tree should have a root");
            local_stack = vec![root];
        }
        let mut me = PreOrderIterator { stack: vec![] };

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
        if !self.stack.is_empty() {
            Some(self.stack.remove(0))
        } else {
            None
        }
    }
}

impl DoubleEndedIterator for PreOrderIterator {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.stack.pop()
    }
}
