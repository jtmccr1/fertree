use std::path::Iter;
use std::collections::HashMap;
use std::sync::mpsc::channel;

pub type TreeIndex = usize;

pub struct TreeNode {
    pub annotation: Option<usize>,
    pub parent: Option<TreeIndex>,
    pub first_child: Option<TreeIndex>,
    pub next_sibling: Option<TreeIndex>,
    pub previous_sibling: Option<TreeIndex>,

}

impl TreeNode {
    pub fn new(annotation: Option<usize>,
               parent: Option<TreeIndex>) -> Self {
        TreeNode { annotation: annotation, parent: parent, first_child: None, next_sibling: None, previous_sibling: None }
    }
}

pub struct Tree {
    nodes: Vec<Option<TreeNode>>,
    root: Option<TreeIndex>,

}

impl Tree {
    pub fn new() -> Self {
        Tree {
            nodes: Vec::new(),
            root: None,
        }
    }
    pub fn iter(&self) -> PreorderIter {
        PreorderIter::new(self.root)
    }
    pub fn set_root(&mut self, root: Option<TreeIndex>) {
        self.root = root
    }
    pub fn add_node(&mut self, node: TreeNode) -> TreeIndex {
        let index = self.nodes.len();
        self.nodes.push(Some(node));
        let child = self.node_at(index).expect("no way we hit this");
        if let Some(parent)=child.parent{
            self.add_child(parent, index);
        }
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
    pub fn remove_node_at(&mut self, index: TreeIndex) -> Option<TreeNode> {
        if let Some(node) = self.nodes.get_mut(index) {
            node.take()
        } else {
            None
        }
    }
    pub fn node_at(&self, index: TreeIndex) -> Option<&TreeNode> {
        return if let Some(node) = self.nodes.get(index) {
            node.as_ref()
        } else {
            None
        };
    }
    pub fn node_at_mut(&mut self, index: TreeIndex) -> Option<&mut TreeNode> {
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
        return children
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
    pub fn next(&mut self, tree: &Tree) -> Option<TreeIndex> {
        while let Some(node_index) = self.stack.pop() {
            if let Some(node) = tree.node_at(node_index) {
                if let Some(sibling)=node.next_sibling{
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


