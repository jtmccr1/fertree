#[derive(Debug)]
pub struct FixedNode {
    pub children: Vec<Box<FixedNode>>,
    pub label: Option<String>,
    pub taxon: Option<String>,
    pub length: Option<f64>,
}


impl FixedNode {
    pub fn new() -> Self {
        FixedNode {
            children: vec![],
            label: None,
            taxon: None,
            length: None,
        }
    }

    pub fn iter(&self) -> PreorderIter {
        PreorderIter::new(self.as_ref())
    }
}

struct PreorderIter<'a> {
    stack: Vec<&'a FixedNode>
}

impl<'a> PreorderIter<'a> {
    fn new(node: &FixedNode) -> Self {
        PreorderIter { stack: vec![node] }
    }
}

impl<'a> Iterator for PreorderIter<'a> {
    type Item = &'a FixedNode;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.stack.pop() {
            for child in node.children.iter() {
                self.stack.push(&**child);
            }
            return Some(&node);
        }
        return None;
    }
}
