
mod introductions{
    use rebl::tree::fixed_tree::FixedNode;
    struct TaxaIntroductionLabel{
        taxa:String,
        introduction:usize,
        tmrca:f64
    }
    fn label_introductions(tree:FixedNode){

    }
}

mod thin{
}

mod collapse{
    use rebl::tree::mutable_tree::{MutableTree, TreeIndex};
    use std::collections::HashSet;
    use rand::Rng;
    use rebl::io::parser::newick_parser::AnnotationValue;


    pub fn collapse_uniform_clades(tree:&MutableTree,key:&String) {
        // Get clades that are all the same
        let mut types: Vec<HashSet<&String>> = Vec::with_capacity(tree.get_node_count());
        for node_ref in tree.preorder_iter().rev() {
            let mut set = HashSet::new();
            if tree.get_num_children(node_ref) == 0 {
                if let Some(annotation) = tree.get_annotation(node_ref, key) {
                    match annotation {
                        AnnotationValue::Discrete(s) => {
                            set.insert(s);
                            types[node_ref] = set;
                        },
                        _ => { panic!("not a discrete trait") }
                    }
                } else {
                    //TODO ignore missing
                }
            } else {
                let mut i = 0;
                while i < tree.get_num_children(node_ref) {
                    if let Some(child) = tree.get_child(node_ref, i) {
                        set = set.union(&types[child]).cloned().collect();
                        i += 1;
                    }
                }
                types[node_ref] = set;
            }
        }
    }


    fn pick_random_tip(tree: &MutableTree, node: TreeIndex)->TreeIndex {
        let kids = tree.get_num_children(node);
        if kids==0{
            return node;
        }
        let next_kid = rand::thread_rng().gen_range(0..kids);
        return pick_random_tip(tree,tree.get_child(node, next_kid).expect("child out of range"))
    }
}

mod annotate{
    use rebl::tree::mutable_tree::MutableTree;
    use std::collections::HashMap;
    use rebl::io::parser::newick_parser::AnnotationValue;


    pub fn annotate_tips(mut tree:MutableTree, annotation_map:HashMap<String,HashMap<String,AnnotationValue>>){
        for taxon in annotation_map.keys(){
            let node_ref = tree.get_taxon_node(taxon).expect(&*("Taxon ".to_owned() + taxon + " not found in tree"));
            if let Some(annotations)=annotation_map.get(taxon){
                for (key,value) in annotations{
                    tree.annotate_node(node_ref, key.clone(), value.clone())
                }
            }
        }
    }
}

mod split{
    
}