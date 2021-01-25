use std::path::Iter;
use std::option::Option;
use super::fixed_tree::FixedNode;

pub type TreeIndex = usize;


pub struct MutableTreeNodeReference {
    taxon: Option<String>,
    parent: Option<TreeIndex>,
}

pub struct MutableTreeNode {
    pub taxon: Option<String>,
    pub parent: Option<TreeIndex>,
    pub first_child: Option<TreeIndex>,
    pub next_sibling: Option<TreeIndex>,
    pub previous_sibling: Option<TreeIndex>,
    pub branch_length: Option<f64>,
    pub number:usize
}

impl MutableTreeNode {
    fn new(taxon: Option<String>,
               parent: Option<TreeIndex>,
                number:usize) -> Self {
        MutableTreeNode { taxon: taxon, parent: parent, first_child: None, next_sibling: None, previous_sibling: None, branch_length: None,number }
    }
}

pub struct MutableTree {
    pub nodes: Vec<Option<MutableTreeNode>>,
    pub external_nodes:Vec<Option<TreeIndex>>,
    pub interal_nodes:Vec<Option<TreeIndex>>,
    root: Option<TreeIndex>,

}

impl MutableTree {
    pub fn new(root:FixedNode)->Self{
       let mut tree = MutableTree {
            nodes: Vec::new(),
            external_nodes:Vec::new(),
            interal_nodes:Vec::new(),
            root: None,
        };

        tree.new_helper(root, None);
        tree.set_root(Some(0));
        return tree

    }
    fn new_helper(&mut self, node:FixedNode,parent:Option<TreeIndex>){
        let index = self.nodes.len();
        self.add_node(MutableTreeNode::new(node.taxon,parent,index));
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

    //TODO split out number
    fn add_node(&mut self, node: MutableTreeNode) -> TreeIndex {
        let index = self.nodes.len();
        self.nodes.push(Some(node));
        let child = self.node_at(index).expect("no way we hit this");
        if let Some(parent) = child.parent {
            self.add_child(parent, index);
        }
        return index;
    }
    fn add_tip(&mut self, node: MutableTreeNode)->TreeIndex{
        let index = self.add_node(node);
        self.external_nodes.push(Some(index));
        return index;
    }

    fn add_internal_node(&mut self, node: MutableTreeNode)->TreeIndex{
        let index = self.add_node(node);
        self.interal_nodes.push(Some(index));
        return index;
    }


    pub fn add_child(&mut self, parent: TreeIndex, child: TreeIndex) {
        let mut children = self.get_children(parent);
        if let Some(last_child) = children.pop() {
            let sibling = self.node_at_mut(last_child).expect("sibling not tree");
            sibling.next_sibling = Some(child);
            let child_node = self.node_at_mut(child).expect("child not in tree");
            child_node.previous_sibling = Some(last_child);
        } else {
            let mut parent_node = self.node_at_mut(parent).expect("parent to be part of the tree");
            parent_node.first_child = Some(child);
        }
    }
    pub fn set_parent(&mut self, parent: TreeIndex, child: TreeIndex) {
        let node = self.node_at_mut(child).expect("Node not in tree");
        node.parent = Some(parent);
    }
    pub fn remove_node_at(&mut self, index: TreeIndex) -> Option<MutableTreeNode> {
        if let Some(node) = self.nodes.get_mut(index) {
            node.take()
            //TODO remove from external node list and or internal node list
        } else {
            None
        }
    }
    pub fn node_at(&self, index: TreeIndex) -> Option<&MutableTreeNode> {
        return if let Some(node) = self.nodes.get(index) {
            node.as_ref()
        } else {
            None
        };
    }
    pub fn node_at_mut(&mut self, index: TreeIndex) -> Option<&mut MutableTreeNode> {
        return if let Some(node) = self.nodes.get_mut(index) {
            node.as_mut()
        } else {
            None
        };
    }
    pub fn get_children(&self, index: TreeIndex) -> Vec<TreeIndex> {
        let mut children = Vec::new();
        let mut node = self.node_at(index).expect("Node not in tree");
        if let Some(fist_child) = node.first_child {
            children.push(fist_child);
            let mut child = self.node_at(fist_child).expect("Node not in tree");
            if let Some(sibling) = child.next_sibling {
                children.push(sibling);
                child = self.node_at(sibling).expect("expected sibling node to be in the tree");
            }
        }
        return children;
    }

    pub fn set_branchlength(&mut self, index: TreeIndex, bl: f64) {
        let node = self.node_at_mut(index).expect("node not in tree");
        node.branch_length = Some(bl);
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
            if let Some(node) = tree.node_at(node_index) {
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



