use std::option::Option;
use super::fixed_tree::FixedNode;
use std::collections::HashMap;
use crate::parsers::newick_parser::AnnotationValue;
use std::collections::hash_map::Keys;

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
    pub annotations:HashMap<String,AnnotationValue>,
    number: usize,

}

impl MutableTreeNode {
    pub(crate) fn new(taxon: Option<String>,
                      number:usize) -> Self {
        MutableTreeNode { taxon: taxon, label: None, parent: None, first_child: None, next_sibling: None, previous_sibling: None, length: None, annotations: HashMap::new(), number }
    }
}

pub struct MutableTree {
    nodes: Vec<Option<MutableTreeNode>>,
    external_nodes:Vec<Option<TreeIndex>>,
    internal_nodes:Vec<Option<TreeIndex>>,
    annotation_type:HashMap<String,AnnotationValue>,
    root: Option<TreeIndex>,
}

impl MutableTree {
    pub fn new(root: FixedNode) ->Self{
       let mut tree = MutableTree {
           nodes: Vec::new(),
           external_nodes:Vec::new(),
           internal_nodes:Vec::new(),
           annotation_type: HashMap::new(),
           root: None,
        };
        tree.new_helper(root, None);
        tree.set_root(Some(0));
        return tree
    }
    fn new_helper(&mut self, node: FixedNode, parent:Option<TreeIndex>){
        let index = self.nodes.len();
        let new_node = MutableTreeNode::new(node.taxon,index);
        self.nodes.push(Some(new_node));

        if let Some(length)=node.length{
            self.set_length(index, length);
        }

        if let Some(mut annotation_map) = node.annotations {
            for (key,value) in annotation_map.into_iter() {
                    self.annotate_node(index, key, value);
            }
        }

        if node.children.len()>0{
            self.internal_nodes.push(Some(index));
        }else{
            self.external_nodes.push(Some(index));
        }
        if let Some(p) = parent{
            self.add_child(p,index);
            self.set_parent(p,index);
        }
        for child in node.children.into_iter(){
            self.new_helper(*child, Some(index));
        }
    }

    pub fn iter(&self) -> PreorderIter {
        PreorderIter::new(self.root)
    }
    pub fn set_root(&mut self, root: Option<TreeIndex>) {
        self.root = root
    }

    pub fn add_child(&mut self, parent: TreeIndex, child: TreeIndex) {
        let parent_node = self.get_node_mut(parent).expect("Node not in tree");
        if let Some(first_child)=parent_node.first_child{
            let mut sibling_node = self.get_node_mut(first_child).expect("Node not in tree");
            while let Some(sibling) = sibling_node.next_sibling {
                sibling_node = self.get_node_mut(sibling).expect("expected sibling node to be in the tree");
            }
            sibling_node.next_sibling=Some(child);
            let previous_sib_i=sibling_node.number;
            let mut child_node = self.get_node_mut(child).expect("node in tree");
            child_node.previous_sibling = Some(previous_sib_i);
        }
        else {
            let mut parent_node = self.get_node_mut(parent).expect("parent to be part of the tree");
            parent_node.first_child = Some(child);
        }
    }

    pub fn remove_child(&mut self,parent:TreeIndex,child:TreeIndex){
        let mut successful_removal = true;
        let child_node = self.get_unwrapped_node(child);
       if let Some(previous_slibling_i)=child_node.previous_sibling{
           if let Some(next_sibling_i)=child_node.next_sibling{
               let mut prev_sib = self.get_unwrapped_node_mut(previous_slibling_i);
               prev_sib.next_sibling = Some(next_sibling_i);
               let mut next_sib = self.get_unwrapped_node_mut(next_sibling_i);
               next_sib.previous_sibling = Some(previous_slibling_i);
           }else{
               let mut prev_sib = self.get_unwrapped_node_mut(previous_slibling_i);
               prev_sib.next_sibling=None;
           }
       }else{
           let  parent_node = self.get_unwrapped_node_mut(parent);
            if let Some(first_child)=parent_node.first_child{
                if first_child==child{
                    let child_node = self.get_unwrapped_node(child);
                    if let Some(next_sibling_i)=child_node.next_sibling{
                        let  parent_node = self.get_unwrapped_node_mut(parent);
                        parent_node.first_child = Some(next_sibling_i);
                        let mut next_sib = self.get_unwrapped_node_mut(next_sibling_i);
                        next_sib.previous_sibling=None;
                    }else{
                        let  parent_node = self.get_unwrapped_node_mut(parent);
                        parent_node.first_child=None;
                    }
                }else{
                    println!("not a child of the parent node! TODO make this a warning");
                    successful_removal=false;
                }
            }
       }

        if successful_removal {
            let mut_child = self.get_unwrapped_node_mut(child);
            mut_child.next_sibling = None;
            mut_child.previous_sibling=None;
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
    fn get_unwrapped_node(&self,index:TreeIndex)->&MutableTreeNode{
        return self.get_node(index).expect("node not in tree")
    }
    fn get_unwrapped_node_mut(&mut self,index:TreeIndex)->&mut MutableTreeNode{
        return self.get_node_mut(index).expect("node not in tree")
    }

    pub fn get_node_count(&self)->usize{
        self.nodes.len()
    }
    pub fn get_internal_node_count(&self)->usize{
        self.internal_nodes.len()
    }
    pub fn get_external_node_count(&self)->usize{
        self.external_nodes.len()
    }

    pub fn get_num_children(&mut self, node_ref:TreeIndex) ->TreeIndex{
        let mut count = 0;
        let parent_node = self.get_unwrapped_node(node_ref);
        if let Some(first_child)=parent_node.first_child{
            let  mut sibling_node = self.get_unwrapped_node_mut(first_child);
            while let Some(sibling) = sibling_node.next_sibling {
                 sibling_node = self.get_unwrapped_node_mut(sibling);
                count+=1;
            }
        }
        return count;
    }
    pub fn get_child(&mut self, node_ref:TreeIndex, index:usize) ->Option<TreeIndex>{
        let mut count = 0;
        let parent_node = self.get_unwrapped_node(node_ref);
        if let Some(first_child)=parent_node.first_child{
            if index==0 {
                return Some(first_child);
            }
            let  mut sibling_node = self.get_unwrapped_node_mut(first_child);
            while let Some(sibling) = sibling_node.next_sibling {
                sibling_node = self.get_unwrapped_node_mut(sibling);
                count+=1;
                if count==index{
                    return Some(sibling);
                }
            }
        }
        return None;
    }
    pub fn get_next_sibling(&self,index:TreeIndex)->Option<TreeIndex>{
        let node = self.get_unwrapped_node(index);
        if let Some(sib)=node.next_sibling{
            return Some(sib);
        }
        return None;
    }
    pub fn get_previous_sibling(&self,index:TreeIndex)->Option<TreeIndex>{
        let node = self.get_unwrapped_node(index);
        if let Some(sib)=node.previous_sibling{
            return Some(sib);
        }
        return None;
    }
    pub fn set_length(&mut self, index: TreeIndex, bl: f64) {
        let node = self.get_node_mut(index).expect("node not in tree");
        node.length = Some(bl);
    }
    pub fn get_annotation(&self, index: TreeIndex, key: &str)->Option<&AnnotationValue> {
        return  self.get_unwrapped_node(index).annotations.get(key)
    }
    pub fn get_annotation_keys(&self) -> Keys<'_, String, AnnotationValue> {
        self.annotation_type.keys()
    }
    pub fn get_root(&self)->Option<TreeIndex>{
        self.root
    }
    pub fn annotate_node(&mut self, index: TreeIndex, key: String, value: AnnotationValue) {

        if let Some(annotation)=self.annotation_type.get(&key){
            let value_type = std::mem::discriminant(&value);
            let annotation_type = std::mem::discriminant(annotation);
          if value_type==annotation_type {
              let mut node = self.get_unwrapped_node_mut(index);
              node.annotations.insert(key, value);
          }else{
                panic!("tried to annotate node with an missmatched annotation type");
            }
        }else{
            match value{
                AnnotationValue::Discrete(_) => {
                    self.annotation_type.insert(key.clone(),AnnotationValue::Discrete("".to_string()));

                }
                AnnotationValue::Continuous(_) =>  {
                    self.annotation_type.insert(key.clone(),AnnotationValue::Continuous(0.0));

                }
                AnnotationValue::Set(_) =>  {
                    //TODO check internal types
                    self.annotation_type.insert(key.clone(),AnnotationValue::Set(vec![AnnotationValue::Discrete("0".to_string())]));

                }
            }
            let mut node = self.get_unwrapped_node_mut(index);
            node.annotations.insert(key, value);
        }

    }
    pub fn label_node(&mut self, index: TreeIndex, label: String) {
        let  node = self.get_unwrapped_node_mut(index);
        node.label = Some(label);
    }

    pub fn get_label(&self,index:TreeIndex)->&Option<String>{
        let node = self.get_unwrapped_node(index);
        &node.label
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



