use std::path::Iter;
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::channel;

pub type TreeIndex = usize;

#[derive(Debug)]
pub struct FixedNode {
    pub children: Vec<Box<FixedNode>>,
    pub label: Option<String>,
    pub taxon: Option<String>,
    pub length: Option<f64>
}

impl FixedNode {
    pub(crate) fn new() ->Self{
        FixedNode {
            children: vec![],
            label: None,
            taxon:None,
            length:None
        }
    }
}


pub struct TreeNode {
    pub taxon: Option<String>,
    pub parent: Option<TreeIndex>,
    pub first_child: Option<TreeIndex>,
    pub next_sibling: Option<TreeIndex>,
    pub previous_sibling: Option<TreeIndex>,
    pub branch_length: Some(f64),
}

impl TreeNode {
    pub fn new(annotation: Option<String>,
               parent: Option<TreeIndex>) -> Self {
        TreeNode { taxon: annotation, parent: parent, first_child: None, next_sibling: None, previous_sibling: None, branch_length: None }
    }
}

pub struct Tree {
    pub nodes: Vec<Option<TreeNode>>,
    root: Option<TreeIndex>,

}

impl Tree {
    pub fn new() -> Self {
        Tree {
            nodes: Vec::new(),
            root: None,
        }
    }
    pub fn from_fixed_node(root:FixedNode){

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
        if let Some(parent) = child.parent {
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
        return children;
    }

    pub fn set_branchlength(&mut self, index: TreeIndex, bl: f64) {
        let node = self.node_at_mut(index).expect("node not in tree");
        node.branch_length = bl;
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


// pub struct NewickParser {
//     node_stack: Vec<TreeIndex>,
//     current_node: Option<TreeIndex>,
//     in_comment: bool,
//     in_quote: bool,
//     expectation: Expectation,
//     deliminators: HashSet<&'static str>,
//
// }
//
// impl NewickParser {
//     pub fn new() -> Self {
//         Self {
//             node_stack: vec![],
//             current_node: None,
//             in_comment: false,
//             in_quote: false,
//             expectation: Expectation::Non,
//             deliminators: vec![",", ":", "[", "]", ";", ":", "(", ")", "{", "}", "\'"].into_iter().collect(),
//         }
//     }
//     fn expect(&mut self, expectation: Expectation) {
//         self.expectation = expectation;
//     }
//     fn clear_expectation(&mut self) {
//         self.expectation = Expectation::Non;
//     }
//     pub fn parse_newick(&mut self, string: String) -> Tree {
//         let nwk_string = string.split_word_bounds().collect::<Vec<&str>>();
//         let mut tree = Tree::new();
//         for token in nwk_string {
//             if self.deliminators.contains(token) {
//                 match token {
//                     //TODO figure out what to do here
//                     "(" => {
//                         let node = TreeNode::new(None, self.current_node);
//                         if let Some(node) = self.current_node {
//                             self.node_stack.push(n)
//                         }
//                         self.current_node = Some(tree.add_node(node));
//                         self.expect(Expectation::Taxon);
//                     }
//                     "," => {
//                         let parent = self.node_stack.pop();
//                         tree.set_parent(parent, self.current_node.unwrap());
//                         tree.add_child(parent.unwrap(), self.current_node.unwrap());
//                         self.expect(Expectation::Taxon);
//                     }
//                     ")" => {
//                         let parent = self.node_stack.pop();
//                         tree.set_parent(parent, self.current_node.unwrap());
//                         tree.add_child(parent.unwrap(), self.current_node.unwrap());
//                         self.current_node = parent;
//                         self.expect(Expectation::Label);
//                     }
//                     ":" => {
//                         self.expect(Expectation::Length)
//                     }
//                     _ => {}
//                 }
//                 println!("found {}", token)
//             } else {
//                 match self.expectation {
//                     Expectation::Label => {
//                         self.clear_expectation();
//                     }
//                     Expectation::Length => {
//                         tree.set_branchlength(tree.get_node(self.current_node.unwrap()).unwrap(), token.parse().unwrap());
//                         self.clear_expectation();
//                     }
//                     Expectation::Taxon => {
//                         let taxon: Taxon = Taxon { name: token.parse().unwrap() };
//                         tree.set_taxon(self.current_node.unwrap(), Some(taxon));
//                         self.clear_expectation()
//                     }
//                     Expectation::AnnotationKey => {}
//                     Expectation::AnnotationValue => {}
//                     Expectation::Non => {
//                         println!("AAHHHH got here : {}", token)
//                     }
//                 }
//             }
//         };
//         println!("{}", tree.nodes.len());
//         return tree;
//     }
// }

