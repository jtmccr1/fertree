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