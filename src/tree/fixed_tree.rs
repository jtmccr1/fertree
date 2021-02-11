use super::AnnotationValue;
use std::collections::HashMap;

#[derive(Debug)]
pub struct FixedNode {
    pub children: Vec<FixedNode>,
    pub label: Option<String>,
    pub taxon: Option<String>,
    pub length: Option<f64>,
    pub annotations: Option<HashMap<String, AnnotationValue>>,
}

impl Default for FixedNode {
    fn default() -> Self {
        Self::new()
    }
}

impl FixedNode {
    pub fn new() -> Self {
        FixedNode {
            children: vec![],
            label: None,
            taxon: None,
            length: None,
            annotations: None,
        }
    }

    pub fn iter(&self) -> PreorderIter {
        PreorderIter::new(&self)
    }
}

pub struct PreorderIter<'a> {
    stack: Vec<&'a FixedNode>,
}

impl<'a> PreorderIter<'a> {
    fn new(node: &'a FixedNode) -> Self {
        PreorderIter { stack: vec![node] }
    }
}

impl<'a> Iterator for PreorderIter<'a> {
    type Item = &'a FixedNode;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.stack.pop() {
            for child in node.children.iter() {
                self.stack.push(child);
            }
            return Some(&node);
        };
        None
    }
}
